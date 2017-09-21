$(document).ready(function(){
    $('#img-delete').click(function(){
    // The way you should do this:
    // put the data-href attributes on the actual list items
    // pull them using this.getAttribute rather than jQuery.
    // Make the requests simply by taking data-href as the url
    // and data-method as the http method. Unfortunately
    // the only way to do this is to stringly type it...
        $.ajax($('.wide-img').attr("data-href"), 
            {
                contentType: 'application/json',
                method: 'delete',
                data: get_image_request_data(this),
                success: function(){console.log("done");}
            }
        );
    });
});

function get_image_request_data(elem) {
    return "";
}
