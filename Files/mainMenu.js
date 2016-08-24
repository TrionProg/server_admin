var mainMenuIsOpen=false;
var consoleIsOpen=false;
var logoutButtonIsOpen=false;

var serverButtonState="run";

function showLogoutButton(){
    if( !logoutButtonIsOpen ){
        logoutButtonIsOpen=true;
        document.getElementById("logoutButton").style.display="block";

        var timer = setInterval(function() {
            document.getElementById("logoutButton").style.top="10px";

            clearInterval(timer);
        }, 20);
    }
}

function hideLogoutButton(){
    if( logoutButtonIsOpen ){
        logoutButtonIsOpen=false;
        document.getElementById("logoutButton").style.top="-40px";

        var timer = setInterval(function() {
            document.getElementById("logoutButton").style.display="none";
            clearInterval(timer);
        }, 300);
    }
}

function openMainMenu(){
    if( !mainMenuIsOpen ){
        mainMenuIsOpen=true;
        document.getElementById("mainMenu").style.display="block";

        if( consoleIsOpen ){
            document.getElementById("consoleLog").style.display="block";
            document.getElementById("consoleCloseButton").style.display="block";
        }else{
            showLogoutButton();
        }

        var timer = setInterval(function() {
            var mainMenu=document.getElementById("mainMenu");
            mainMenu.style.opacity=1;

            clearInterval(timer);
        }, 20);
    }
}

function closeMainMenu(){
    if( mainMenuIsOpen ){
        document.getElementById("mainMenu").style.left="-100%";

        var timer = setInterval(function() {
            mainMenu=document.getElementById("mainMenu");
            mainMenu.style.display="none";
            mainMenu.style.left="0px";
            mainMenu.style.opacity=0;
            mainMenuIsOpen=false;

            clearInterval(timer);
        }, 500);
    }
}

function showConsole(){
    if (!consoleIsOpen && mainMenuIsOpen) {
        consoleIsOpen=true;
        var consoleLog=document.getElementById("consoleLog");
        consoleLog.style.display="block";

        var timer = setInterval(function() {
            var consoleLog=document.getElementById("consoleLog");
            consoleLog.style.height="calc(100% - 232px)";

            var mainMenuButtonsBox=document.getElementById("mainMenuButtonsBox");
            mainMenuButtonsBox.style.top="10px";

            hideLogoutButton();

            var timerCCB = setInterval(function() {
                document.getElementById("consoleCloseButton").style.display="block";
                document.getElementById("consoleCloseButton").style.bottom="calc(100% - 196px)";

                clearInterval(timerCCB);
            }, 500);

            clearInterval(timer);
        }, 20);
    }
}

function startConsole(){
    console.log("start");
    var consoleCloseButton=document.getElementById("consoleCloseButton");
    consoleCloseButton.style.bottom="calc(100% - 195px)";
    consoleCloseButton.style.display="block";
}

function hideConsole(){
    if( consoleIsOpen ){
        document.getElementById("consoleCloseButton").style.display="none";

        document.getElementById("mainMenuButtonsBox").style.top="calc(50% - 59px)";

        var consoleLog=document.getElementById("consoleLog");
        consoleLog.style.height="0px";

        var timer = setInterval(function() {
            document.getElementById("consoleLog").style.display="none";
            consoleIsOpen=false;

            clearInterval(timer);
        }, 500);

        showLogoutButton();
    }
}

function setServerButtonState( state ){
    document.getElementById("serverButton_run").style.display="none";
    document.getElementById("serverButton_process").style.display="none";
    document.getElementById("serverButton_stop").style.display="none";

    switch (state) {
        case "run":
            document.getElementById("serverButton").style.backgroundColor="#448eb5";
            document.getElementById("serverButton").style.borderColor="#eea87b";
            document.getElementById("serverButton_run").style.display="block";

            break;
        case "process":
            document.getElementById("serverButton").style.backgroundColor="#5b448d";
            document.getElementById("serverButton").style.borderColor="#c19d85";
            document.getElementById("serverButton_process").style.display="block";

            var timer = setInterval(function() {
                setServerButtonState( "stop" );

                clearInterval(timer);
            }, 5000);

            break;
        case "stop":
            document.getElementById("serverButton").style.backgroundColor="#a65cbe";
            document.getElementById("serverButton").style.borderColor="#c19d85";
            document.getElementById("serverButton_stop").style.display="block";

            break;
    }

    if( state=="run" ){
        document.getElementById("mapManagerButton").style.backgroundColor="#b1dadc";
        document.getElementById("mapManagerButton").style.borderColor="#eea87b";

        document.getElementById("modManagerButton").style.backgroundColor="#a6dd6a";
        document.getElementById("modManagerButton").style.borderColor="#733a17";

        document.getElementById("settingsButton").style.backgroundColor="#c7d8e2";
        document.getElementById("settingsButton").style.borderColor="#7c6658";
    }else{
        document.getElementById("mapManagerButton").style.backgroundColor="#dadada";
        document.getElementById("mapManagerButton").style.borderColor="#8a7d75";

        document.getElementById("modManagerButton").style.backgroundColor="#dadada";
        document.getElementById("modManagerButton").style.borderColor="#8a7d75";

        document.getElementById("settingsButton").style.backgroundColor="#dadada";
        document.getElementById("settingsButton").style.borderColor="#8a7d75";
    }

    serverButtonState=state;
}

function mainMenuButtonOver( buttonName ){
    switch (buttonName) {
        case "serverButton":
            switch (serverButtonState) {
                case "run":
                    document.getElementById("serverButton").style.backgroundColor="#1c506c";
                    break;
                case "stop":
                    document.getElementById("serverButton").style.backgroundColor="#6424c7";
                    break;
            }

            break;
        case "mapManagerButton":
            if( serverButtonState=="run" ){
                document.getElementById("mapManagerButton").style.backgroundColor="#85aac0";
            }

            break;
        case "modManagerButton":
            if( serverButtonState=="run" ){
                document.getElementById("modManagerButton").style.backgroundColor="#49d62e";
            }

            break;
        case "settingsButton":
            if( serverButtonState=="run" ){
                document.getElementById("settingsButton").style.backgroundColor="#85aac0";
            }

            break;
    }
}

function mainMenuButtonOut( buttonName ){
    switch (buttonName) {
        case "serverButton":
            switch (serverButtonState) {
                case "run":
                    document.getElementById("serverButton").style.backgroundColor="#448eb5";
                    break;
                case "stop":
                    document.getElementById("serverButton").style.backgroundColor="#a65cbe";
                    break;
            }

            break;
        case "mapManagerButton":
            if( serverButtonState=="run" ){
                document.getElementById("mapManagerButton").style.backgroundColor="#b1dadc";
            }

            break;
        case "modManagerButton":
            if( serverButtonState=="run" ){
                document.getElementById("modManagerButton").style.backgroundColor="#a6dd6a";
            }

            break;
        case "settingsButton":
            if( serverButtonState=="run" ){
                document.getElementById("settingsButton").style.backgroundColor="#c7d8e2";
            }

            break;
    }
}

function serverButtonClick(){
    switch (serverButtonState) {
        case "run":
            var xhr = new XMLHttpRequest();
            xhr.open('GET', '/gui:run server', true);
            xhr.send();

            var timer = setInterval(function() {
                setServerButtonState( "process" );

                clearInterval(timer);
            }, 1000);

            break;
        case "stop":
            var xhr = new XMLHttpRequest();
            xhr.open('GET', '/gui:stop server', true);
            xhr.send();

            var timer = setInterval(function() {
                setServerButtonState("run");
                openErrorMenu( "Server error" );

                clearInterval(timer);
            }, 3000);

            break;
    }
}

function inputCommand( event ){
    if( event.keyCode == 13 ){
        var input=document.getElementById("consoleInput").value;
        document.getElementById("consoleInput").value="";

        if( input=="exit" ){
            //document.getElementById("consoleLog").value+="No no. You can not stop admin server remotely.\nYou should use \"logout\" command to do you guess =)\n\n\
            //If you really want to stop admin server, you need stop it by command line interface on host, kill command, or contact with host administration\n";
        }else{
            var sodium=window.sodium;

            var jsonData=JSON.stringify({
                adminKey:adminKey,
                source:"console",
                commands:input
            });

            var cipherData=sodium.crypto_secretbox_easy(jsonData,requestNonce,requestKey,"base64");

            var xhr = new XMLHttpRequest();
            xhr.open('POST', '/cmd', true);
            xhr.send(cipherData);

            xhr.onreadystatechange = function() {
                if( xhr.readyState != 4 ) return;

                if( xhr.status==200 ){
                    loadNews(xhr.responseText);
                }else{
                    console.log( xhr.status + ': ' + xhr.statusText + ':' + xhr.responseText );
                }
            }
        }

        return false;
    }
}
