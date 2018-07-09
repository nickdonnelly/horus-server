hljs.configure({tabReplace: '    ',});
hljs.initHighlightingOnLoad();
$(document).ready(function(){
    $('#img-delete').click(function(){
        let locstr = '/manage/';
        locstr += $(this).attr('data-loc');
        locstr += '/0';
        console.log(locstr);
        $.ajax($(this).attr("data-href"), 
            {
                contentType: 'application/json',
                method: 'delete',
                success: function(){ window.location.href = locstr;}
            }
        );
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
