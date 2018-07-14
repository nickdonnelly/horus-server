hljs.configure({tabReplace: '    ',});
hljs.initHighlightingOnLoad();
document.addEventListener("DOMContentLoaded", function(event){
    var colors = [
      "#3949ab", "#39ab9c", // blue
      "#43a047", "#70a043", // green
      "#8BC34A", "#c3c14a", // light green
        "#9C27B0", "#5727b0" // purple 
    ];
    var color = colors[Math.floor(Math.random()*colors.length)];
    var body = document.getElementsByTagName("body")[0];
    body.style.background = color;

    let pw_form = document.getElementById("password-form");
    let data_obj = document.getElementById("type-data");
    let loader_img = document.getElementById("loader-img");
    let password_field = document.getElementById("password-field");
    pw_form.addEventListener('submit', function(e){
        e.preventDefault();
        let request = new XMLHttpRequest();
        request.open("POST", "/password/" + pw_form.getAttribute("data-id"), true);
        request.setRequestHeader("horus-resource-type", data_obj.getAttribute("data-type"));
        request.onreadystatechange = function(state){
            if(request.readyState == XMLHttpRequest.DONE) {
                loader_img.style.display = "none";
                if(request.status != 200) {
                    shake();
                } else {
                    let resource_url = request.responseText;
                    setResourceURL(resource_url);
                    pw_form.style.display = 'none';
                    document.getElementById("password-box").style.display = 'none';
                }
            }
        };
        let submitted_password = password_field.value;
        loader_img.style.display = 'inline-block';
        request.send(submitted_password);
    });

    function shake() {
        let original_bg = password_field.style.background;
        password_field.style.transform = "translateX(-20px)";
        password_field.style.background = "#dd5435";
        setTimeout(() => { password_field.style.transform = "translateX(20px)"; }, 200);
        setTimeout(() => { 
            password_field.style.transform = "translateX(0)"; 
            password_field.style.background = original_bg;
        }, 200);
    }

    function setResourceURL(url) {
        switch(data_obj.getAttribute("data-type")){
            case "image":
            case "video":
                console.log(data_obj);
                data_obj.src = url;
                break;
            case "file":
                data_obj.href = url;
                break;
        }
    }

});


