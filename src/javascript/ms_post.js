<script>
async function ms_post(endpoint, body) {
    let url = "https://localhost" + endpoint;

    const response = await fetch(url, {
        method: "POST",
        headers: {
            "Content-Type": "application/json"
        },
        credentials: "same-origin",
        body: JSON.stringify(body),
    });

    return response;
}
</script>