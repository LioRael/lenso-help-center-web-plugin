pub(crate) const PAGE: &str = r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <meta name="color-scheme" content="light">
  <meta name="description" content="Search published help or contact support.">
  <title>Lenso Help Center</title>
  <link rel="stylesheet" href="/help/assets/app.css">
</head>
<body>
  <header class="topbar">
    <a class="brand" href="/help" aria-label="Help Center home"><span aria-hidden="true">L</span><b>Help Center</b></a>
    <a class="quiet-link" href="#check-request">Check a request</a>
  </header>
  <main>
    <section class="hero" aria-labelledby="hero-title">
      <p class="eyebrow">SELF-SERVICE SUPPORT</p>
      <h1 id="hero-title">What can we help you solve?</h1>
      <p class="lede">Search our published guides first. If they do not answer your question, send the same context to the support team.</p>
      <form id="search-form" class="search" role="search">
        <label class="sr-only" for="query">Search for an answer</label>
        <input id="query" name="query" type="search" required maxlength="240" autocomplete="off" placeholder="Search for an answer">
        <button type="submit">Search</button>
      </form>
      <p id="search-message" class="status" role="status" aria-live="polite"></p>
    </section>

    <section id="results-section" class="content-section hidden" aria-labelledby="results-title">
      <div class="section-heading"><div><p class="eyebrow">PUBLISHED GUIDES</p><h2 id="results-title">Possible answers</h2></div><span id="result-count" class="count"></span></div>
      <div id="results" class="result-list"></div>
    </section>

    <article id="article" class="article hidden" aria-live="polite">
      <button id="article-back" class="text-button" type="button">← Back to results</button>
      <p class="eyebrow">HELP ARTICLE</p>
      <h2 id="article-title"></h2>
      <p id="article-date" class="article-date"></p>
      <div id="article-body" class="article-body"></div>
      <a class="quiet-link" href="#contact-support">Still need help? Contact support</a>
    </article>

    <section id="contact-support" class="support-grid" aria-labelledby="contact-title">
      <div class="support-intro">
        <p class="eyebrow">STILL STUCK?</p>
        <h2 id="contact-title">Send the problem to a person</h2>
        <p>Sign in, then tell us what you expected, what happened, and anything you already tried. We will return a reference you can use to check the response.</p>
        <div class="promise"><span>1</span><p><b>One clear request</b><br>No support dashboard or agent tooling exposed.</p></div>
        <div class="promise"><span>2</span><p><b>A private, durable reference</b><br>Your signed-in identity and reference reveal only your public conversation.</p></div>
      </div>
      <form id="support-form" class="card">
        <label>Bearer access token <span>(required for support, never sent to article search)</span><input name="access_token" type="password" required autocomplete="off"></label>
        <label>What do you need help with?<input name="title" maxlength="240" required></label>
        <label>What happened?<textarea name="description" rows="7" maxlength="12000" required placeholder="Include the steps you took and what you expected to happen."></textarea></label>
        <p id="support-error" class="form-error" role="alert"></p>
        <button type="submit">Send support request</button>
        <small>The Auth Plugin verifies your token and passes a short-lived signed assertion to Support Intake.</small>
      </form>
      <div id="receipt" class="card receipt hidden" role="status" aria-live="polite">
        <div class="success-mark" aria-hidden="true">✓</div>
        <p class="eyebrow">REQUEST RECEIVED</p>
        <h2 id="receipt-title"></h2>
        <p>Keep this reference. You will need it with the same signed-in identity to check the reply.</p>
        <button id="receipt-ref" class="reference" type="button" title="Copy reference"></button>
        <a class="button-link" href="#check-request">Check this request</a>
      </div>
    </section>

    <section id="check-request" class="check-section" aria-labelledby="check-title">
      <div><p class="eyebrow">FOLLOW UP</p><h2 id="check-title">Check a support request</h2><p>Use the same signed-in identity and your request reference.</p></div>
      <form id="status-form" class="status-form"><label>Bearer access token<input name="access_token" type="password" required autocomplete="off"></label><label>Request reference<input name="case_ref" maxlength="240" required placeholder="SUP-…"></label><button type="submit">Check status</button></form>
      <p id="status-error" class="form-error" role="alert"></p>
      <article id="case-status" class="case-status hidden" aria-live="polite"><div class="case-heading"><div><span id="case-state" class="badge"></span><h3 id="case-title"></h3><p id="case-ref"></p></div><time id="case-updated"></time></div><p id="case-description" class="case-description"></p><div id="messages" class="messages"></div><form id="attachment-form" class="attachment-form"><div><b>Add a public text attachment</b><p>One UTF-8 text/plain file, up to 8 MiB. PNG, JPEG, and other binary uploads are not available in this release.</p></div><input id="attachment-file" name="file" type="file" accept="text/plain,.txt,.log" required><button type="submit">Upload text file</button><p id="attachment-error" class="form-error" role="alert"></p></form></article>
    </section>
  </main>
  <footer><span>Lenso Help Center</span><span>Published answers first. Human support when needed.</span></footer>
  <div id="toast" role="status" aria-live="polite"></div>
  <script src="/help/assets/app.js" defer></script>
</body>
</html>"##;

pub(crate) const CSS: &str = r#":root{font-family:Inter,ui-sans-serif,-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;color:#17211f;background:#f4f6f2;--ink:#17211f;--muted:#66716e;--line:#dce2dc;--paper:#fff;--accent:#156b5d;--accent-dark:#0c5146;--soft:#e3f0eb;--error:#a13a32}*{box-sizing:border-box}html{scroll-behavior:smooth}body{margin:0;background:radial-gradient(circle at 50% -10%,#d9ede6 0,transparent 34rem),#f4f6f2}.topbar{height:68px;max-width:1120px;margin:auto;padding:0 28px;display:flex;align-items:center;justify-content:space-between}.brand{display:flex;align-items:center;gap:10px;color:var(--ink);text-decoration:none}.brand span{display:grid;place-items:center;width:34px;height:34px;border-radius:10px;background:var(--ink);color:white;font-weight:800}.quiet-link{color:var(--accent);font-weight:650;text-underline-offset:3px}main{max-width:1040px;margin:auto;padding:38px 28px 90px}.hero{text-align:center;padding:58px 0 76px;max-width:800px;margin:auto}.eyebrow{margin:0 0 10px;color:var(--accent);font-size:11px;font-weight:800;letter-spacing:.16em}.hero h1{font-size:clamp(38px,6vw,64px);letter-spacing:-.055em;line-height:1.02;margin:0}.lede{max-width:650px;margin:22px auto 30px;color:var(--muted);font-size:17px;line-height:1.6}.search{display:grid;grid-template-columns:1fr auto;gap:9px;max-width:680px;margin:auto;padding:8px;background:var(--paper);border:1px solid #d2dbd4;border-radius:16px;box-shadow:0 18px 50px #2d4e4218}.search input{border:0;padding:12px 13px;font-size:16px;background:transparent;outline:none}.search button{min-width:110px}.status{min-height:20px;color:var(--muted);font-size:13px}.content-section,.article,.check-section{background:var(--paper);border:1px solid var(--line);border-radius:18px;padding:28px;box-shadow:0 12px 40px #2d4e420b;margin-bottom:42px}.section-heading{display:flex;align-items:flex-end;justify-content:space-between;border-bottom:1px solid var(--line);padding-bottom:18px}.section-heading h2,.support-intro h2,.check-section h2,.article h2{font-size:30px;letter-spacing:-.035em;margin:0}.count{color:var(--muted);font-size:12px}.result-list{display:grid}.result{display:grid;grid-template-columns:1fr auto;gap:18px;width:100%;padding:20px 4px;text-align:left;background:none;border:0;border-bottom:1px solid var(--line);color:var(--ink)}.result:last-child{border:0}.result:hover h3{color:var(--accent)}.result h3{margin:0 0 7px;font-size:17px}.result p{margin:0;color:var(--muted);font-size:12px}.result>span{align-self:center;color:var(--accent);font-size:20px}.empty-results{text-align:center;padding:38px;color:var(--muted)}.article{padding:42px;max-width:820px;margin:0 auto 42px}.article h2{font-size:38px}.article-date{color:var(--muted);font-size:12px;border-bottom:1px solid var(--line);padding-bottom:22px}.article-body{white-space:pre-wrap;line-height:1.75;margin:28px 0 34px;font-family:inherit;color:#293431}.text-button{padding:0 0 28px;background:none;color:var(--accent)}.support-grid{scroll-margin-top:26px;display:grid;grid-template-columns:.8fr 1.2fr;gap:48px;padding:58px 0}.support-intro>p:not(.eyebrow){color:var(--muted);line-height:1.65}.promise{display:flex;gap:13px;margin-top:24px}.promise span{flex:0 0 30px;height:30px;display:grid;place-items:center;border-radius:50%;background:var(--soft);color:var(--accent);font-weight:750}.promise p{margin:0;color:var(--muted);font-size:13px;line-height:1.55}.promise b{color:var(--ink)}.card{display:grid;gap:17px;padding:28px;background:var(--paper);border:1px solid var(--line);border-radius:18px;box-shadow:0 16px 45px #2d4e4210}.field-row{display:grid;grid-template-columns:1fr 1fr;gap:13px}label{display:grid;gap:7px;font-size:12px;font-weight:700}label span,small{font-weight:400;color:var(--muted)}input,textarea{width:100%;border:1px solid #ccd5cf;border-radius:9px;padding:11px 12px;font:inherit;color:var(--ink);background:#fbfcfa;outline:none}input:focus,textarea:focus{border-color:var(--accent);box-shadow:0 0 0 3px #156b5d18}button,.button-link{border:0;border-radius:9px;padding:11px 15px;background:var(--accent);color:white;font:inherit;font-weight:720;cursor:pointer;text-decoration:none;text-align:center}button:hover,.button-link:hover{background:var(--accent-dark)}button:disabled{opacity:.6;cursor:wait}.form-error{min-height:18px;margin:0;color:var(--error);font-size:12px}.receipt{text-align:center;place-content:center}.success-mark{width:46px;height:46px;display:grid;place-items:center;margin:0 auto;border-radius:50%;background:var(--soft);color:var(--accent);font-size:23px}.receipt h2{margin:0;font-size:24px}.receipt p:not(.eyebrow){color:var(--muted);line-height:1.6}.reference{font-family:ui-monospace,SFMono-Regular,Menlo,monospace;background:#eef3ef;color:var(--ink);border:1px dashed #aebbb3}.reference:hover{background:#e5ece7}.check-section{scroll-margin-top:26px;margin-top:50px}.check-section>div:first-child{margin-bottom:24px}.check-section>div:first-child p:last-child{color:var(--muted)}.status-form{display:grid;grid-template-columns:1fr 1fr auto;align-items:end;gap:12px}.case-status{margin-top:28px;padding-top:25px;border-top:1px solid var(--line)}.case-heading{display:flex;justify-content:space-between;gap:20px}.case-heading h3{font-size:22px;margin:9px 0 5px}.case-heading p,.case-heading time{color:var(--muted);font-size:12px}.badge{display:inline-flex;padding:5px 9px;background:var(--soft);border-radius:999px;color:var(--accent);font-size:10px;font-weight:800;text-transform:uppercase;letter-spacing:.07em}.case-description{padding:15px;border-radius:10px;background:#f6f8f5;white-space:pre-wrap;line-height:1.55}.messages{display:grid;gap:10px;margin-top:18px}.message{max-width:80%;padding:13px 15px;border:1px solid var(--line);border-radius:12px;background:#f7f9f6}.message.you{justify-self:end;background:var(--soft);border-color:#cce2d9}.message header{display:flex;justify-content:space-between;gap:20px;font-size:10px;text-transform:uppercase;letter-spacing:.08em;color:var(--muted)}.message p{white-space:pre-wrap;margin:8px 0 0;line-height:1.55;font-size:13px}.attachment-form{display:grid;grid-template-columns:1fr auto;gap:10px;align-items:end;margin-top:26px;padding:18px;border:1px solid var(--line);border-radius:12px;background:#f8faf7}.attachment-form>div,.attachment-form .form-error{grid-column:1/-1}.attachment-form b{font-size:13px}.attachment-form p{margin:4px 0 0;color:var(--muted);font-size:11px}.attachment-form input[type=file]{font-size:12px}footer{max-width:1040px;margin:auto;padding:24px 28px 40px;border-top:1px solid var(--line);display:flex;justify-content:space-between;color:var(--muted);font-size:12px}.hidden{display:none!important}.sr-only{position:absolute;width:1px;height:1px;padding:0;margin:-1px;overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap;border:0}#toast{position:fixed;right:22px;bottom:22px;opacity:0;transform:translateY(12px);padding:11px 15px;border-radius:9px;background:var(--ink);color:white;font-size:12px;transition:.2s;pointer-events:none}#toast.show{opacity:1;transform:none}@media(max-width:760px){main{padding:10px 18px 60px}.topbar{padding:0 18px}.hero{padding:44px 0 58px}.search{grid-template-columns:1fr}.search button{width:100%}.support-grid{grid-template-columns:1fr;gap:25px}.field-row,.status-form,.attachment-form{grid-template-columns:1fr}.content-section,.article,.check-section{padding:22px}.article h2{font-size:30px}.case-heading,footer{display:block}.case-heading time{display:block;margin-top:10px}.message{max-width:95%}}"#;

pub(crate) const JS: &str = r"const $=selector=>document.querySelector(selector);
function requestId(){return crypto.randomUUID()}
async function api(path,options={},token=''){const headers={'Content-Type':'application/json',...(options.headers||{})};if(token)headers.Authorization=`Bearer ${token}`;const response=await fetch(path,{...options,headers});if(!response.ok){let problem={};try{problem=await response.json()}catch{}throw new Error(problem.detail||`Request failed (${response.status})`)}return response.json()}
function busy(form,value){form.querySelectorAll('button,input,textarea').forEach(element=>element.disabled=value)}
function toast(message){const element=$('#toast');element.textContent=message;element.classList.add('show');setTimeout(()=>element.classList.remove('show'),2200)}
function date(value){try{return new Intl.DateTimeFormat(undefined,{dateStyle:'medium',timeStyle:'short'}).format(new Date(value))}catch{return value}}
function element(tag,className,text){const node=document.createElement(tag);if(className)node.className=className;if(text!==undefined)node.textContent=text;return node}

const searchForm=$('#search-form');
searchForm.addEventListener('submit',async event=>{event.preventDefault();const message=$('#search-message');message.textContent='Searching published guides…';busy(searchForm,true);try{const query=new FormData(searchForm).get('query').trim();const result=await api('/api/help/articles/search',{method:'POST',body:JSON.stringify({query,limit:8})});renderResults(result.articles);message.textContent=result.articles.length?`Found ${result.articles.length} possible ${result.articles.length===1?'answer':'answers'}.`:'No published guide matched. Send the details below and a person can help.';$('#contact-support').classList.toggle('attention',result.articles.length===0)}catch(error){message.textContent=error.message}finally{busy(searchForm,false)}});

function renderResults(articles){const section=$('#results-section');const host=$('#results');host.replaceChildren();$('#article').classList.add('hidden');section.classList.remove('hidden');$('#result-count').textContent=`${articles.length} ${articles.length===1?'result':'results'}`;if(!articles.length){host.append(element('p','empty-results','No published answer matched this search.'));return}articles.forEach(article=>{const button=element('button','result');button.type='button';const copy=element('div');copy.append(element('h3','',article.title),element('p','',`Published ${date(article.published_at)}`));button.append(copy,element('span','', '→'));button.addEventListener('click',()=>openArticle(article.article_ref));host.append(button)})}

async function openArticle(articleRef){const message=$('#search-message');message.textContent='Opening article…';try{const article=await api(`/api/help/articles/${encodeURIComponent(articleRef)}`,{headers:{}});$('#article-title').textContent=article.title;$('#article-date').textContent=`Published ${date(article.published_at)}`;$('#article-body').textContent=article.body_markdown;$('#results-section').classList.add('hidden');$('#article').classList.remove('hidden');$('#article').scrollIntoView({behavior:'smooth',block:'start'});message.textContent=''}catch(error){message.textContent=error.message}}
$('#article-back').addEventListener('click',()=>{$('#article').classList.add('hidden');$('#results-section').classList.remove('hidden');$('#results-section').scrollIntoView({behavior:'smooth'})});

const supportForm=$('#support-form');
supportForm.addEventListener('submit',async event=>{event.preventDefault();$('#support-error').textContent='';busy(supportForm,true);try{const values=Object.fromEntries(new FormData(supportForm));const token=values.access_token;delete values.access_token;const receipt=await api('/api/help/support',{method:'POST',body:JSON.stringify({...values,idempotency_key:requestId()})},token);$('#receipt-title').textContent=receipt.title;$('#receipt-ref').textContent=receipt.case_ref;supportForm.classList.add('hidden');$('#receipt').classList.remove('hidden');const statusForm=$('#status-form');statusForm.elements.access_token.value=token;statusForm.elements.case_ref.value=receipt.case_ref;toast('Support request received')}catch(error){$('#support-error').textContent=error.message}finally{busy(supportForm,false)}});
$('#receipt-ref').addEventListener('click',async event=>{try{await navigator.clipboard.writeText(event.currentTarget.textContent);toast('Reference copied')}catch{toast('Select the reference to copy it')}});

const statusForm=$('#status-form');
const currentRequest={caseRef:'',token:''};
statusForm.addEventListener('submit',async event=>{event.preventDefault();$('#status-error').textContent='';busy(statusForm,true);try{const values=Object.fromEntries(new FormData(statusForm));const token=values.access_token;const status=await api('/api/help/support/status',{method:'POST',body:JSON.stringify({case_ref:values.case_ref})},token);currentRequest.caseRef=status.case_ref;currentRequest.token=token;renderStatus(status)}catch(error){$('#case-status').classList.add('hidden');$('#status-error').textContent=error.message}finally{busy(statusForm,false)}});
function renderStatus(value){$('#case-state').textContent=value.state.replaceAll('_',' ');$('#case-title').textContent=value.title;$('#case-ref').textContent=value.case_ref;$('#case-updated').textContent=`Updated ${date(value.updated_at)}`;$('#case-description').textContent=value.description;const host=$('#messages');host.replaceChildren();if(!value.messages.length){host.append(element('p','empty-results','No public replies yet.'))}else{value.messages.forEach(item=>{const message=element('article',`message ${item.author}`);const header=element('header');header.append(element('b','',item.author==='you'?'You':'Support'),element('time','',date(item.created_at)));message.append(header,element('p','',item.body));host.append(message)})}$('#case-status').classList.remove('hidden')}

const attachmentForm=$('#attachment-form');
attachmentForm.addEventListener('submit',async event=>{event.preventDefault();$('#attachment-error').textContent='';const file=$('#attachment-file').files[0];if(!file)return;if(file.size===0||file.size>8*1024*1024){$('#attachment-error').textContent='Choose a non-empty text file no larger than 8 MiB.';return}busy(attachmentForm,true);try{const bytes=new Uint8Array(await file.arrayBuffer());new TextDecoder('utf-8',{fatal:true}).decode(bytes);const content=bytesToBase64(bytes);await api('/api/help/support/attachments',{method:'POST',body:JSON.stringify({case_ref:currentRequest.caseRef,message_id:null,filename:file.name,content_type:'text/plain',content,visibility:'public',idempotency_key:requestId()})},currentRequest.token);attachmentForm.reset();toast('Text attachment uploaded')}catch(error){$('#attachment-error').textContent=error instanceof TypeError?'The file must contain valid UTF-8 text.':error.message}finally{busy(attachmentForm,false)}});
function bytesToBase64(bytes){let binary='';for(let offset=0;offset<bytes.length;offset+=32768)binary+=String.fromCharCode(...bytes.subarray(offset,offset+32768));return btoa(binary)}
";
