 var loginMenuIsOpen=false;

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

         var timerCCB = setInterval(function() {
             document.getElementById("loginMenu").style.display="none";
             loginMenuIsOpen=false;

             clearInterval(timerCCB);
         }, 300);

         loginMenu.style.top="-100%";
         loginMenu.style.opacity=0.5;
     }
 }

 function setupLoginButton(){
     var loginButton=document.getElementById("loginButton");
     loginButton.innerHTML="Login";

     loginButton.onclick = function () { checkLogin() };
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

 function checkLogin(){
     var loginButton=document.getElementById("loginButton");

     loginButton.onclick="";
     loginButton.onmouseover="";
     loginButton.onmouseout="";

     loginButton.style.borderColor="#252572";
     loginButton.style.backgroundColor="#385295";
     /*
     loginButton.onclick="";
     loginButton.innerHTML="<div class=\"loginProgressBackground\"><div id=\"loginProgress\" class=\"loginProgress\"></div></div>";

     loginProgressMoveRight();
     */

     var xhr = new XMLHttpRequest();
     xhr.open('GET', '/login', true);
     xhr.send();

     xhr.onreadystatechange = function() {
         if (xhr.readyState != 4) return;

         if (xhr.status != 200) {
             console.log( xhr.status + ': ' + xhr.statusText );
             loginError("Connection problems");
         } else {
             if (xhr.responseText=="accepted\n") {
                 loginSuccess();
             }else{
                 loginError(xhr.responseText);
             }
         }
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
 }
