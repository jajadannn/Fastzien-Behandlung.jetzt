var titelname = ["Faszienbehandlung - Thilo Seifried", "Faszienbehandlung Buchen"]; 
var i = 0;

setInterval(function() { 
    
    document.getElementById("titel").innerHTML = titelname[i];
    i = (i + 1) % titelname.length; 

    }, 10000); // Ändert den Titel alle 2 Sekunden

onclick="Calendly.initPopupWidget({url:'https://calendly.com/termin-faszienbehandlung/jetzt'}); return false;"