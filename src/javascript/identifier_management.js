<script>

async function ban() {
    const ban_identifier = document.getElementById("ban_identifier").value;

    if (ban_identifier.value == "") {
        return;
    }

    if (!confirm("Are you sure you wish to ban this identifier")) {
        return;
    }

    const ban_reason = document.getElementById("ban_reason").value;

    let player_ban = {
        identifier: ban_identifier,
        reason: ban_reason,
        unban_timestamp: null,
    };

    const unban_time = document.getElementById("ban_unbandate");

    if (unban_time.valueAsNumber != NaN ) {
        let time_in_sec = unban_time.valueAsNumber / 1000;
        if ( time_in_sec > 0 ) {
            player_ban.unban_timestamp = time_in_sec;
        }
    }

    const response = await ms_post("/panel/ban", player_ban);
    let response_msg = document.getElementById("ban_result");

    if (response.status != 200) {
        let err_msg = await response.text();
        response_msg.innerText = "Ban failed: " + err_msg;
    } else {
        response_msg.innerText = "Banned";
    }
}

async function check_identifier() {
    const ban_search_identifier = document.getElementById("identifier");

    if (ban_search_identifier.value == "") {
        return;
    }

    let identifier = {
        identifier: ban_search_identifier.value,
    };

    const response = await (await ms_post("/panel/ban_search", identifier)).json();
    let table = document.getElementById("ban_list_table_body");

    table.innerHTML = "";

    for(let i = 0; i < response.length; i++) {            
        let row = table.insertRow();
        row.insertCell(0).innerText = response[i]["identifier"];
        
        if  (response[i]["banned_on"] != null) {
            row.insertCell(1).innerText = new Date(response[i]["banned_on"]).toLocaleString();
        } else {
            row.insertCell(1).innerText = "None";
        }

        if  (response[i]["unban_date"] != null) {
            row.insertCell(2).innerText = new Date(response[i]["unban_date"]).toLocaleString();
        } else {
            row.insertCell(2).innerText = "None";
        }

        row.insertCell(3).innerText = response[i]["reason"]

        let button = document.createElement("button");
        button.innerText = "Unban";
        button.id = response[i]["ban_id"];
        button.addEventListener("click", unban);
        row.insertCell(4).append(button);
    }
}

async function unban(button) {
    
    let target = {
        key: Number(button.target.id)
    };

    const response = await ms_post("/panel/unban", target);
    
    if (response.status == 200) {
        check_identifier()
    }
}

</script>
