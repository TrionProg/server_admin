 var loginMenuIsOpen=false;

 var loginKeysB="";
 var loginPublicKeyA="";
 var loginNonce="";

 function openLoginMenu(){
     if(! loginMenuIsOpen ){
         loginMenuIsOpen=true;
         document.getElementById("loginMenu").style.display="block";

         document.getElementById("loginError").style.display="none";

         var loginPassword=document.getElementById("loginPassword");
         loginPassword.innerHTML="";
         loginPassword.style.backgroundColor="#D0D0D0";
         loginPassword.style.borderColor="#707070";

         setupLoginButton();

         var timer = setInterval(function() {
             var loginMenu = document.getElementById("loginMenu");

             loginMenu.style.top="0px";
             loginMenu.style.opacity=1;

             clearInterval(timer);
         }, 20);
     }
 }

 function closeLoginMenu(){
     if( loginMenuIsOpen ){
         var loginMenu = document.getElementById("loginMenu");
         loginMenu.style.top="-100%";
         loginMenu.style.opacity=0.5;

         var timer = setInterval(function() {
             document.getElementById("loginMenu").style.display="none";
             loginMenuIsOpen=false;

             clearInterval(timer);
         }, 300);
     }
 }

 function setupLoginButton(){
     var loginButton=document.getElementById("loginButton");
     loginButton.innerHTML="Login";
     loginButton.style.borderColor="#1616bf";
     loginButton.style.backgroundColor="#0c3bb7";

     loginButton.onclick = function () { loginStep1() };
     loginButton.onmouseover = function () {
         var loginButton=document.getElementById("loginButton");
         loginButton.style.borderColor="#252572";
         loginButton.style.backgroundColor="#385295";
     }

     loginButton.onmouseout = function () {
         var loginButton=document.getElementById("loginButton");
         loginButton.style.borderColor="#1616bf";
         loginButton.style.backgroundColor="#0c3bb7";
     }
 }

function loginStep1(){
    if( !cryptoLibIsReady ){
        document.getElementById("errorsList").value="Crypto library sodium.js is not ready to use yet.\nPlease, wait or reload the page";
        openErrorMenu("Error");
        return;
    }

    //===================Disactive Login Button==================
    var loginButton=document.getElementById("loginButton");

    loginButton.onclick="";
    loginButton.onmouseover="";
    loginButton.onmouseout="";

    loginButton.style.borderColor="#252572";
    loginButton.style.backgroundColor="#385295";

    //====================Login process==========================
    var sodium=window.sodium;

    loginKeysB = sodium.crypto_box_keypair();
    loginNonce = sodium.randombytes_buf(sodium.crypto_box_NONCEBYTES);

    var requestBody="id:{0}\npublic key b:{1}\nnonce:{2}\n".format(
        "0",
        sodium.to_base64(loginKeysB.publicKey),
        sodium.to_base64(loginNonce)
    );

    var xhr = new XMLHttpRequest();
    xhr.open('POST', '/login', true);
    xhr.send(requestBody);

    xhr.onreadystatechange = function() {
        if( xhr.readyState != 4 ) return;

        if( xhr.status==200 ){
            var colonPos=xhr.responseText.indexOf(":");
            if( colonPos>0 ){
                var status=xhr.responseText.slice(0, colonPos);
                var content=xhr.responseText.slice(colonPos+1);

                switch (status) {
                    case "ok":
                        loginStep2(content);
                        return;
                    case "message":
                        loginError(content);
                        break;
                    case "error":
                        document.getElementById("errorsList").value=msg.slice(1)+".\nPlease, try again.";
                        openErrorMenu("Error");
                        break;
                    default:
                        document.getElementById("errorsList").value="Error has been occured. See javascript log for details.\n\nPlease, try again";
                        openErrorMenu("Error");
                }
            }
        }

        console.log( xhr.status + ': ' + xhr.statusText + ':' + xhr.responseText );
        setupLoginButton();
    }
}

function loginStep2(responseText) {
    var sodium=window.sodium;

    var idAndKey=responseText.split(";");
    loginId=idAndKey[0];
    loginPublicKeyA=new Uint8Array( sodium.from_base64(idAndKey[1]) );

    requestKey=sodium.randombytes_buf(sodium.crypto_secretbox_KEYBYTES);
    requestNonce=sodium.randombytes_buf(sodium.crypto_secretbox_NONCEBYTES);

    responseKey=sodium.randombytes_buf(sodium.crypto_secretbox_KEYBYTES);
    responseNonce=sodium.randombytes_buf(sodium.crypto_secretbox_NONCEBYTES);

    var jsonData=JSON.stringify({
        password:document.getElementById("loginPassword").value,
        requestKey:sodium.to_base64(requestKey),
        requestNonce:sodium.to_base64(requestNonce),
        responseKey:sodium.to_base64(responseKey),
        responseNonce:sodium.to_base64(responseNonce)
    });

    var cipherData=sodium.crypto_box_easy(jsonData,loginNonce,loginPublicKeyA,loginKeysB.privateKey,"base64");

    var requestBody=requestBody="id:{0}\ncipher data:{1}\n".format(
        loginId,
        cipherData
    );

    var xhr = new XMLHttpRequest();
    xhr.open('POST', '/login', true);
    xhr.send(requestBody);

    xhr.onreadystatechange = function() {
        if( xhr.readyState != 4 ) return;

        if( xhr.status==200 ){
            var colonPos=xhr.responseText.indexOf(":");
            if( colonPos>0 ){
                var status=xhr.responseText.slice(0, colonPos);
                var content=xhr.responseText.slice(colonPos+1);

                switch (status) {
                    case "ok":
                        var cipherAdminKey=new Uint8Array( sodium.from_base64(content) );
                        adminKey=sodium.crypto_secretbox_open_easy(cipherAdminKey, responseNonce, responseKey, "text");

                        loginKeysB="";
                        loginPublicKeyA="";
                        loginNonce="";

                        loginSuccess();
                        return;
                    case "message":
                        loginError(content);
                        break;
                    case "error":
                        document.getElementById("errorsList").value=msg.slice(1)+".\nPlease, try again.";
                        openErrorMenu("Error");
                        break;
                    default:
                        document.getElementById("errorsList").value="Error has been occured. See javascript log for details.\n\nPlease, try again";
                        openErrorMenu("Error");
                }
            }
        }

        console.log( xhr.status + ': ' + xhr.statusText + ':' + xhr.responseText );
        setupLoginButton();
    }
}

 /*
 function loginProgressMoveLeft(){
     document.getElementById("loginProgress").style.left="-1px";
     document.addEventListener('transitionend', function() {
         loginProgressMoveRight();
     });
 }

 function loginProgressMoveRight(){
     document.getElementById("loginProgress").style.left="90px";
     document.addEventListener('transitionend', function() {
         loginProgressMoveLeft();
     });
 }
 */

 function loginError( msg ){
     var loginError=document.getElementById("loginError");
     loginError.innerHTML=msg;
     loginError.style.display="table-cell";

     var loginPassword=document.getElementById("loginPassword");
     loginPassword.value="";
     loginPassword.style.backgroundColor="#e4aeae";
     loginPassword.style.borderColor="#821212";

     setupLoginButton();
 }

 function loginSuccess(){
     document.getElementById("loginError").style.display="none";

     var loginPassword=document.getElementById("loginPassword");
     loginPassword.value="";
     loginPassword.style.backgroundColor="#73D962";
     loginPassword.style.borderColor="#1C8220";

     var timer = setInterval(function() {
         closeLoginMenu();
         openMainMenu();

         clearInterval(timer);
     }, 1000);

     newsTimerInterval=200;
     newsTimer=setInterval(function() {
         checkNews();
     }, newsTimerInterval);
 }
