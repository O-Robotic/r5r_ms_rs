<script>
async function post_login() {
    const username = document.getElementById("username").value;
    const password = document.getElementById("password").value;

    const login_info = {
        username: username,
        password: password,
    };

    const response = await ms_post("/panel/auth", login_info);

    if (response.status != 200) {
        const status_box = document.getElementById("login_message");
        status_box.innerText = "Login Failure"
    }

}
</script>