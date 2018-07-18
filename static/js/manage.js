hljs.configure({tabReplace: '    ',});
hljs.initHighlightingOnLoad();
$(document).ready(function(){
    $('#img-delete').click(function(){
        let locstr = '/manage/';
        locstr += $(this).attr('data-loc');
        locstr += '/0';
        $.ajax($(this).attr("data-href"), 
            {
                contentType: 'application/json',
                method: 'delete',
                success: function(){ window.location.href = locstr;}
            }
        );
    });

    $('#img-pw').click(function(){
        let pw_popup = document.getElementById("password-popup");
        pw_popup.style.display = "flex";
        pw_popup.style.opacity = 1;
    });

    $('#password-submit').click(function(){
        let password_field = document.getElementById("password-field");
        if(password_field.value.length === 0){
            password_field.classList.add("bad-input");
            setTimeout(() => { password_field.classList.remove("bad-input"); }, 300);
            return;
        }

        let elem = $('#img-pw');

        $.ajax(elem.attr('data-href'),
            {
                method: elem.attr('data-method'),
                contentType: 'text/plain',
                data: password_field.value,
                beforeSend: (xhr) => {
                    xhr.setRequestHeader("horus-resource-type", elem.attr('data-type'));
                },
                success: () => { 
                    location.reload();
                }
            });

    });

    $('.is_date').each(function() {
        let datestr = $(this).text();
        datestr = datestr.split(".")[0].replace("T", " ");
        $(this).text(datestr);
    });
    
    $('.paste-row').click(function(){window.location.href = $(this).attr('data-href');});
    $('.paste-data').focusout(function(){
        update_paste();
    });


    $('.close-btn').click(function(){
        let toClose = this.closest(".popup");
        toClose.style.opacity = 0;
        setTimeout(() => { toClose.style.display = "none"; }, 200);
    });

    let upload_dialog = $('#upload-dialog');

    upload_dialog.on('drop', function(e){
        e.preventDefault();
        e.dataTransfer = e.originalEvent.dataTransfer;
        if(e.dataTransfer.items) {
            showLoader();
            for(let i = 0; i < e.dataTransfer.items.length; i++){
                if(e.dataTransfer.items[i].kind == 'file') {
                    let file = e.dataTransfer.items[i].getAsFile();
                    handleFileUpload(e.dataTransfer.items[i].getAsFile());
                }
            }
        }
        upload_dialog.removeClass('active-drop');
        e.dataTransfer.items.clear();
    });

    $('html').on('dragover', function(e) { 
        e.preventDefault(); 
        upload_dialog.addClass("active-drop");
    });

    $('html').on('dragexit', function(e) {
        upload_dialog.removeClass("active-drop");
    });

    function showLoader(){
        $('.loader').css('display', 'inline-block');
    }

    function hideLoader(){
        $('.loader').css('display', 'none');
    }

    function handleFileUpload(file) {
        let reader = new FileReader();
        reader.onload = function(e){
            let file_data = e.target.result;
            switch(file.type) {
                case "image/png":
                    uploadAsImage(file_data, false);
                    break;
                case "video/webm":
                    uploadAsImage(file_data, true);
                    break;
                default:
                    uploadAsFile(file_data);
            }
        };
        // Base64 the data
        reader.readAsDataURL(file);
    }

    function uploadAsFile(file_data) {
        
    }

    function uploadAsImage(file_data, isVideo) {
        let uri = '';
        let ctt = '';
        if(isVideo) {
            uri = '/video/new';
            ctt = 'video/webm';
        } else {
            uri = '/image/new';
            ctt = 'image/png';
        }

        $.ajax(uri, {
            contentType: ctt,
            method: 'POST',
            data: file_data,
            success: function(e) {
                hideLoader();
                notifySuccess('Upload successful!');
            },
            failure: function(e) {
                hideLoader();
                notifyFail('Something went wrong!');
            }
        });
    }

    let notify_box = $('#notify-box');

    function notifySuccess(text) {
        notify_box.removeClass('failure');
        notify_box.addClass('success');
        notify_box.text(text);
        showNotify();
    }

    function notifyFailure(text) {
        notify_box.removeClass('success');
        notify_box.addClass('failure');
        notify_box.css(text);
        showNotify();
    }

    function showNotify() {
        notify_box.css('transform', 'translateY(64px)');
        notify_box.css('opacity', 1);

        setTimeout(function(){
            console.log('retracting');
            notify_box.css('transform', 'none');
            notify_box.css('opacity', 0);
        }, 3000);
    }
    
});

(function($){
   $.fn.innerText = function(msg) {
         if (msg) {
            if (document.body.innerText) {
               for (var i in this) {
                  this[i].innerText = msg;
               }
            } else {
               for (var i in this) {
                  this[i].innerHTML.replace(/&amp;lt;br&amp;gt;/gi,"n").replace(/(&amp;lt;([^&amp;gt;]+)&amp;gt;)/gi, "");
               }
            }
            return this;
         } else {
            if (document.body.innerText) {
               return this[0].innerText;
            } else {
               return this[0].innerHTML.replace(/&amp;lt;br&amp;gt;/gi,"n").replace(/(&amp;lt;([^&amp;gt;]+)&amp;gt;)/gi, "");
            }
         }
   };
})(jQuery);

function update_paste() {
    let title = $('.page-header').text();
    let paste_data = $('.paste-data').innerText();

    $.ajax($('.paste-data').attr('data-href'),
    {
        contentType: 'application/json',
        method: 'put',
        data: JSON.stringify({title: title, paste_data: paste_data, duration_type: 'days', duration_val: -1}),
        success: function(){
            $('.paste-data').removeClass('hljs javascript');
            $('.paste-data').html(paste_data);
            hljs.highlightBlock(document.getElementsByClassName('paste-data')[0]);
            console.log('done');
        },
    });

};

