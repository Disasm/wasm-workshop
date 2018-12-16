import * as wasm from "forth";

//wasm.greet();

//console.log(wasm.interpret("DUP"));

$(document).ready(function(){
    $('#code').bind('input propertychange', function() {
        var code = $("#code").val();
        $("#result").html(wasm.interpret(code));
    });
});
