var errorMenuIsOpen=false;

function openErrorMenu(caption){
    if(! errorMenuIsOpen ){
        errorMenuIsOpen=true;
        document.getElementById("errorMenu").style.display="block";

        document.getElementById("errorCaption").innerHTML=caption;

        var timer = setInterval(function() {
            var errorMenu = document.getElementById("errorMenu");

            errorMenu.style.top="0px";
            errorMenu.style.opacity=1;

            clearInterval(timer);
        }, 20);
    }
}

function closeErrorMenu(){
    if( errorMenuIsOpen ){
        var errorMenu = document.getElementById("errorMenu");
        errorMenu.style.top="-100%";
        errorMenu.style.opacity=0.5;

        var timer = setInterval(function() {
            document.getElementById("errorMenu").style.display="none";
            errorMenuIsOpen=false;

            clearInterval(timer);
        }, 300);
    }
}
