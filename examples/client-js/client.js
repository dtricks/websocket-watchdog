window.onload = function ws() {
  console.log("Starting ws");
  if ("WebSocket" in window) {
    var connection = new WebSocket("ws://127.0.0.1:9002/");
    var text_field = document.getElementById("ws-change");
    var text = "";

    connection.onopen = function () {
      /*Send a small message to the console once the connection is established */
      console.log("Connection open!");
    };
    connection.onclose = function () {
      console.log("Connection closed");
    };
    connection.onerror = function (error) {
      console.log("Error detected: " + error);
    };
    connection.onmessage = function (e) {
      var server_message = e.data;
      text += server_message;
      text_field.innerText = text;
      console.log(server_message);
    };
  } else {
    /*WebSockets are not supported. Try a fallback method like long-polling etc*/
  }
};
