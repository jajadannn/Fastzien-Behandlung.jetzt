/* Local browser checks. Set NODE_PATH to the bundled runtime's node_modules.
   AXE_PATH must point to an integrity-verified axe.min.js; no CDN runs on the site. */
const {chromium} = require('playwright');
const fs = require('node:fs');
const path = require('node:path');
const base = process.argv[2] || 'http://127.0.0.1:8093';
if (!['127.0.0.1','localhost'].includes(new URL(base).hostname)) throw Error('Local preview only');
const out = path.resolve('target/compliance/browser'); fs.mkdirSync(out,{recursive:true});
const axe = path.resolve(process.env.AXE_PATH || 'target/compliance/axe/axe.min.js');
const results = [], external = new Set(), errors = [];
(async () => {
  const browser = await chromium.launch({headless:true,channel:process.env.PLAYWRIGHT_CHANNEL || 'msedge'});
  const publicRoutes = ['/','/faszienbehandlung-ruecken','/preise-und-ablauf','/buchen','/impressum','/datenschutz','/buchungsbedingungen','/barrierefreiheit','/widerruf','/login','/register','/reset-password','/nicht-vorhanden'];
  const privateRoutes = ['/admin','/admin/appointments','/admin/customers','/admin/payments','/admin/faq','/admin/reviews','/admin/settings','/admin/stats','/portal','/portal/book','/portal/profile','/portal/appointments','/portal/credits'];
  for (const width of [1440,390]) {
    const context = await browser.newContext({viewport:{width,height:900},reducedMotion:'reduce',isMobile:width===390,hasTouch:width===390});
    const page = await context.newPage();
    page.on('pageerror',e=>errors.push({width,message:e.message}));
    page.on('request',request=>{const origin=new URL(request.url()).origin;if(origin!==base && !request.url().startsWith('data:')) external.add(origin);});
    for (const route of publicRoutes.concat(privateRoutes)) {
      if (route==='/admin') {
        const response=await context.request.post(base+'/api/auth/login',{headers:{Origin:base},data:{email:'preview@example.test',password:'preview-local-only'}});
        if(!response.ok())throw Error('Preview login failed');
      }
      const response = await page.goto(base+route); await page.waitForTimeout(70);
      await page.addScriptTag({path:axe});
      const audit = await page.evaluate(async () => {
        const axeResult = await axe.run(document,{runOnly:{type:'tag',values:['wcag2a','wcag2aa','wcag21a','wcag21aa','wcag22aa','best-practice']}});
        return {violations:axeResult.violations.map(v=>({id:v.id,impact:v.impact,nodes:v.nodes.map(n=>({target:n.target,summary:n.failureSummary,html:n.html}))})),overflow:document.documentElement.scrollWidth>innerWidth+1};
      });
      results.push({width,route,status:response.status(),...audit});
      if (!process.env.SKIP_SCREENSHOTS && ['/','/buchen','/widerruf','/admin/settings'].includes(route)) await page.screenshot({path:path.join(out,route==='/'?`home-${width}.png`:`${route.slice(1).replaceAll('/','-')}-${width}.png`),fullPage:route!=='/'});
    }
    // Intermediate widths and 320px reflow, using the same rendered content.
    for (const w of [320,768,1024]) for (const route of ['/','/buchen','/widerruf','/portal/profile','/admin/settings']) {
      await page.setViewportSize({width:w,height:900});await page.goto(base+route);
      results.push({width:w,route,layoutOnly:true,overflow:await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth+1)});
    }
    await context.close();
  }
  fs.writeFileSync(path.join(out,'audit.json'),JSON.stringify({results,external:[...external],errors},null,2));
  // Native POST withdrawal works without JavaScript and preserves correction.
  const noJs=await browser.newContext({javaScriptEnabled:false,viewport:{width:390,height:844}});
  const page=await noJs.newPage();await page.goto(base+'/widerruf');
  await page.getByLabel('Vor- und Nachname',{exact:true}).fill('Browser Test');
  await page.getByLabel('E-Mail für die Eingangsbestätigung').fill('browser@example.test');
  await page.getByLabel('Welchen Vertrag oder Teil möchtest du widerrufen?').fill('Testvertrag: Teilwiderruf einer Sitzung');
  await page.getByRole('button',{name:'Widerruf prüfen',exact:true}).click();
  await page.waitForLoadState('load');
  if(!(await page.getByText('Deine Erklärung prüfen',{exact:true}).isVisible())) {fs.writeFileSync(path.join(out,'nojs-failure.html'),await page.content());throw Error('No-JS review missing: '+await page.title());}
  await page.getByRole('button',{name:'Widerruf bestätigen',exact:true}).click();
  if(!(await page.getByText('Dein Widerruf ist eingegangen',{exact:true}).isVisible()))throw Error('No-JS receipt missing');
  if(!process.env.SKIP_SCREENSHOTS)await page.screenshot({path:path.join(out,'withdrawal-receipt-nojs.png'),fullPage:true});
  await noJs.close();await browser.close();
  const report={results,external:[...external],errors,noJsWithdrawal:true};
  fs.writeFileSync(path.join(out,'audit.json'),JSON.stringify(report,null,2));
  const failures=results.filter(r=>r.overflow || r.violations?.length);
  console.log(JSON.stringify({pages:results.length,failures:failures.map(r=>({width:r.width,route:r.route,overflow:r.overflow,violations:r.violations?.map(v=>v.id)})),external:[...external],errors,noJsWithdrawal:true},null,2));
  process.exitCode=failures.length || external.size || errors.length?1:0;
})().catch(e=>{console.error(e);process.exit(1)});
