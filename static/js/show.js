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
});