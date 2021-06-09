window.onload = function ws() {
  console.log("Starting ws");
  if ("WebSocket" in window) {
    var ul_field = document.getElementById("ws-change");
    var list_header = document.getElementById("list-header");
    var badge = document.createElement("span");
    var badge_text = document.createTextNode("");
    badge.appendChild(badge_text);
    list_header.appendChild(badge);
    try {
      var connection = new WebSocket("ws://127.0.0.1:9002/");
    } catch (e) {
      console.log(e);
    }

    connection.onopen = function () {
      /*Send a small message to the console once the connection is established */
      console.log("Connection open!");
      badge.setAttribute("class", "badge bg-success");
      badge_text.textContent = "Connected";
    };
    connection.onclose = function () {
      console.log("Connection closed");
      badge.setAttribute("class", "badge bg-danger");
      badge_text.textContent = "Disconnected";
    };
    connection.onerror = function (error) {
      console.log("Error detected: " + error);
      badge.setAttribute("class", "badge bg-warning");
      badge_text.textContent = "Error";
    };
    connection.onmessage = function (e) {
      var server_message = e.data;
      //console.log(server_message);
      var obj = JSON.parse(server_message);
      //console.log(obj);

      // li
      var li_elem = document.createElement("li");
      li_elem.setAttribute("class", "list-group-item list-group-item-action");

      // h5
      var card_title = document.createElement("h5");
      card_title.appendChild(document.createTextNode(obj["kind"]));
      card_title.setAttribute("class", "mb-1");

      // span badge
      var timestamp = document.createElement("small");
      timestamp.setAttribute("class", "text-muted");
      timestamp.appendChild(document.createTextNode(obj["timestamp"]));

      // small path
      var path = document.createElement("small");
      path.appendChild(document.createTextNode(obj["paths"][0]));

      // div wrapper
      var wrapper = document.createElement("div");
      wrapper.setAttribute("class", "d-flex w-100 justify-content-between");
      wrapper.appendChild(card_title);
      wrapper.appendChild(timestamp);

      // attach
      li_elem.appendChild(wrapper);
      li_elem.appendChild(path);
      ul_field.appendChild(li_elem);
    };
  } else {
    /*WebSockets are not supported. Try a fallback method like long-polling etc*/
  }
};
