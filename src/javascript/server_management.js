<script>
async function kick_button_pressed(button) {
    let players = document.getElementsByName("players_to_kick");
    let uids = []
    
    for (let i = 0; i < players.length; i++) {
        uids.push(Number(players[i].value))
    } 

    const request = {
        server_uid: button.value,
        player_uids: uids
    };

    console.log(request)

    const response = await ms_post("/panel/kick", request);

    let kick_message = document.getElementById("kick_message")

    if (response.status == 200 ) {
        kick_message.innerText = "Selcted player(s) will be kicked on next bulk check"
    } else {
        kick_message.innerText = "Kick request failed"
    }
}
</script>