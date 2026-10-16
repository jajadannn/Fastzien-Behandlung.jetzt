// Shared keyboard/focus behavior, without changing the form's business logic.
(function () {
  document.querySelectorAll('svg.icon').forEach(icon => {
    icon.setAttribute('aria-hidden', 'true'); icon.setAttribute('focusable', 'false');
  });
  const autofill = {first_name:'given-name',last_name:'family-name',phone:'tel',street:'street-address',zip_code:'postal-code',city:'address-level2',new_email:'email',email_password:'current-password',current_password:'current-password',new_password:'new-password'};
  Object.entries(autofill).forEach(([id,value]) => {
    const input = document.getElementById(id);
    if (input) { input.autocomplete = value; if (id === 'zip_code') input.inputMode = 'numeric'; }
  });
  document.querySelectorAll('.form-group').forEach(group => {
    const input = group.querySelector('input,textarea,select');
    const help = group.querySelector('small');
    if (input && help && !input.hasAttribute('aria-describedby')) {
      help.id = help.id || input.id + '-help'; input.setAttribute('aria-describedby',help.id);
    }
  });
  document.querySelectorAll('[role="alert"]').forEach(message => {
    message.tabIndex = -1;
    new MutationObserver(() => {
      if (message.offsetParent && message.textContent.trim()) message.focus({preventScroll:false});
    }).observe(message,{attributes:true,attributeFilter:['style'],childList:true});
  });
  document.querySelectorAll('#reschedule-overlay,.admin-modal-overlay').forEach(overlay => {
    const dialog = overlay.firstElementChild;
    if (!dialog) return;
    const heading = dialog.querySelector('h2,h3');
    dialog.setAttribute('role','dialog'); dialog.setAttribute('aria-modal','true'); dialog.tabIndex=-1;
    if (heading) { heading.id=heading.id || overlay.id+'-title'; dialog.setAttribute('aria-labelledby',heading.id); }
    let open = false, returnTo;
    overlay.setAttribute('aria-hidden','true');
    const isOpen = () => overlay.matches('.admin-modal-overlay') ? overlay.classList.contains('active') : getComputedStyle(overlay).display !== 'none';
    const close = () => { overlay.classList.remove('active'); if(overlay.id==='reschedule-overlay') overlay.style.display='none'; };
    new MutationObserver(() => {
      const visible = isOpen();
      if (visible && !open) {
        returnTo=document.activeElement; open=true;
        overlay.setAttribute('aria-hidden','false');
        Array.from(document.body.children).filter(el=>el!==overlay && !el.contains(overlay)).forEach(el=> { el.dataset.dialogInertBefore=String(el.inert); el.inert=true; });
        const first=dialog.querySelector('input:not([type="hidden"]),button,select,textarea,a[href]');
        (first || dialog).focus();
      } else if (!visible && open) { open=false; overlay.setAttribute('aria-hidden','true'); document.querySelectorAll('[data-dialog-inert-before]').forEach(el=>{el.inert=el.dataset.dialogInertBefore==='true';delete el.dataset.dialogInertBefore;}); if(returnTo && returnTo.isConnected) returnTo.focus(); }
    }).observe(overlay,{attributes:true,attributeFilter:['class','style']});
    overlay.addEventListener('keydown',event => {
      if (event.key==='Escape') { event.preventDefault(); close(); }
      if (event.key==='Tab') {
        const items=Array.from(dialog.querySelectorAll('a[href],button:not(:disabled),input:not([type="hidden"]):not(:disabled),select,textarea,[tabindex="0"]')).filter(el=>el.offsetParent);
        const first=items[0],last=items[items.length-1];
        if (!first) {event.preventDefault();dialog.focus();}
        else if (event.shiftKey && (document.activeElement===first || document.activeElement===dialog)) {event.preventDefault();last.focus();}
        else if (!event.shiftKey && document.activeElement===last) {event.preventDefault();first.focus();}
      }
    });
  });
})();
