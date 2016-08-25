
var sessionKey="";
var sessionNonce="";

var answerKey="";
var answerNonce="";

var adminKey="";
var dialogTimer="";

var consoleText="";
var CONSOLE_TEXT_MAX_LINES=250;
var consoleTextLines=0;

var requestQueue=[];

function printConsole(msg){
    var msgLines=1;
    var pos=0;
    while(true){
        var foundPos=msg.indexOf("\n",pos);
        if( foundPos==-1 ) break;

        msgLines++;
        pos=foundPos+1;
    }

    var consoleLog=document.getElementById("consoleLog");
    var scroll=consoleLog.scrollTop==consoleLog.scrollHeight;

    if( consoleTextLines+msgLines>CONSOLE_TEXT_MAX_LINES ){
        var removeLines=consoleTextLines+msgLines-CONSOLE_TEXT_MAX_LINES;

        var pos=0;
        while(removeLines>0){
            var foundPos=consoleText.indexOf("\n",pos);
            if( foundPos==-1 ) break;

            removeLines-=1;
            pos=foundPos+1;
        }

        consoleText=consoleText.slice(pos)+msg+"\n";
        consoleTextLines=CONSOLE_TEXT_MAX_LINES;
    }else{
        consoleText+=msg+"\n";

        if( consoleTextLines<26 && consoleTextLines+msgLines>=26 ) scroll=true;

        consoleTextLines+=msgLines;
    }

    consoleLog.value=consoleText;

    if( scroll ) consoleLog.scrollTop=consoleLog.scrollHeight;
}

function ask(){
    var sodium=window.sodium;

    var request={
        url:"/arenews",
        data:adminKey,
    };

    if( requestQueue.length>0 ){
        var req=requestQueue.shift();

        request={
            url:req.url,
            data:sodium.crypto_secretbox_easy(req.data,requestNonce,requestKey,"base64")
        };
    }

    var xhr = new XMLHttpRequest();
    xhr.open('POST', request.url, true);
    xhr.send(request.data);

    xhr.onreadystatechange = function() {
        if( xhr.readyState != 4 ) return;

        if( xhr.status==200 ){
            readResponse(xhr.responseText);
        }else if( xhr.status==400 ){
            console.log( xhr.status + ': ' + xhr.statusText + ':' + xhr.responseText );

            if( requestQueue.length==0 ) {
                dialogTimer=setTimeout(function() {ask();}, 100);
            }else{
                ask();
            }
        }
    }
}

function readResponse(responseText){
    var cipherBody=new Uint8Array( sodium.from_base64(responseText) );
    body=sodium.crypto_secretbox_open_easy(cipherBody, responseNonce, responseKey, "text");

    var fields=body.split(";\n");

    for(i=0;i<fields.length;i++){
        var field=fields[i];

        var errorOccurred=false;

        var colonPos=field.indexOf(":");
        if( colonPos>0 ){
            var fieldName=field.slice(0, colonPos);
            var fieldValue=field.slice(colonPos+1);

            if( fieldName=="admin key" ){
                adminKey=fieldValue;
            }else if( fieldName=="log" ){
                printConsole(fieldValue);

                if( errorOccurred ){
                    document.getElementById("errorsList").value+=fieldValue+"\n";
                }
            }else if( fieldName=="game server state" ){
                switch( fieldValue ){
                    case "disactive":
                        setServerButtonState( "start");
                        break;
                    case "starting":
                        setServerButtonState( "process");
                        break;
                    case "working":
                        setServerButtonState( "stop");
                        break;
                    case "stopping":
                        setServerButtonState("process");
                        break;
                    case "error":
                        setServerButtonState( "start");
                        errorOccurred=true;
                        document.getElementById("errorsList").value="";
                        break;
                }
            }

            if( errorOccurred ){
                document.getElementById("errorsList").value+="\n\nSee console for details";
                openErrorMenu( "Server error" );
            }
        }
    }

    if( requestQueue.length==0 ) {
        if( fields.length>2 ){
            dialogTimer=setTimeout(function() {ask();}, 50);
        }else{
            dialogTimer=setTimeout(function() {ask();}, 100);
        }
    }else{
        ask();
    }
}
