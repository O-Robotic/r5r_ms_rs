use {
    actix_session::SessionExt,
    actix_web::{
        body::EitherBody,
        dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
        http, Error, HttpResponse,
    },
    futures::{
        future::{LocalBoxFuture, TryFutureExt},
        FutureExt,
    },
};

pub struct ProtectedEndpoint;

impl<S, B> Transform<S, ServiceRequest> for ProtectedEndpoint
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = ProtectedEndpointMiddleware<S>;
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(ProtectedEndpointMiddleware { service }))
    }
}

pub struct ProtectedEndpointMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for ProtectedEndpointMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;
    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if req.cookie("id").is_some() {
            let session = req.get_session().get::<String>("user");
            if session.is_ok() && session.unwrap().is_some() {
                return self
                    .service
                    .call(req)
                    .map_ok(ServiceResponse::map_into_left_body)
                    .boxed_local();
            }
        }
        let (request, _pl) = req.into_parts();
        println!("Failed auth");
        let response = HttpResponse::Found()
            .insert_header((http::header::LOCATION, "/login"))
            .finish()
            .map_into_right_body();

        Box::pin(async { Ok(ServiceResponse::new(request, response)) })
    }
}
