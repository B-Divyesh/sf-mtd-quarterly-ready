import './styles.css';
import { accountantCsv, hmrcHandoff, parseCsv, pounds, summarise } from './records';
import { createShare, leaveDemo, loadDocument, loadRemote, loadShare, resetDemo, saveDocument } from './storage';
import type { Category, QuarterDocument, Transaction } from './types';

const PRODUCT = 'Quarterly Ready';
const SLUG = 'mtd-quarterly-ready';
const BILLING = `https://api.sociobot.in/api/v1/products/${SLUG}`;
const CATEGORIES: Category[] = ['Sales', 'Rent and rates', 'Travel', 'Office costs', 'Professional fees', 'Repairs', 'Other', ''];
const app = document.querySelector<HTMLDivElement>('#app')!;
let currentDocument: QuarterDocument | null = null;
let currentDemo = false;
let notice = '';

function escapeHtml(value: unknown): string {
  return String(value ?? '').replace(/[&<>'"]/g, character => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', "'": '&#39;', '"': '&quot;' })[character]!);
}

function header(): string {
  return `<header class="site-header">
    <a class="wordmark" href="/" data-link aria-label="Quarterly Ready home"><span class="wordmark-dial" aria-hidden="true"><i></i><i></i><i></i><i></i></span><span>Quarterly<br>Ready</span></a>
    <nav aria-label="Main navigation"><a href="/records" data-link>Records</a><a href="/demo" data-link>Demo</a><a href="/privacy" data-link>Privacy</a></nav>
  </header>`;
}

function footer(): string {
  return `<footer class="site-footer"><p>Checked quarters for UK sole traders.</p><nav aria-label="Footer navigation"><a href="/privacy" data-link>Privacy</a><a href="/terms" data-link>Terms</a><a href="https://hello-factory.sociobot.in" rel="external">Built by Param Factory <span class="sr-only">(external)</span></a></nav><p>Version 1.0 · Original generated imagery</p></footer>`;
}

function layout(main: string, demo = false): string {
  return `${demo ? `<aside class="demo-banner" aria-label="Demo mode"><strong>Demo — sample data, nothing is saved</strong><span><button class="text-button" id="reset-demo">Reset demo</button><a href="/records" data-start-real>Start for real</a></span></aside>` : ''}${header()}${main}${footer()}`;
}

function homePage(): string {
  return layout(`<main id="main">
    <section class="hero">
      <div class="hero-copy">
        <p class="eyebrow">MTD QUARTER CONTROL · UK 2026</p>
        <h1 tabindex="-1">Turn records into a checked quarterly update</h1>
        <p class="lede">For UK sole traders, tutors and landlords who need MTD records without a full accounting suite.</p>
        <div class="hero-action"><a class="primary-button" href="/demo" data-link>Try it with sample data</a><span>Opens a private sample quarter. No account needed.</span></div>
        <ul class="plain-facts"><li><span class="lamp teal"></span>Demo changes never enter your records.</li><li><span class="lamp orange"></span>Accountant pack downloads as a CSV.</li><li><span class="lamp brass"></span>Free quarter. £99 one-time licence.</li></ul>
      </div>
      <figure class="hero-art"><picture><source media="(max-width: 720px)" srcset="/assets/instrument-panel-720.webp"><img src="/assets/instrument-panel.webp" width="1200" height="800" fetchpriority="high" alt="A four-part quarterly dial on a paper bookkeeping machine."></picture><figcaption>One quarter. Four checks. A clear handoff.</figcaption></figure>
    </section>
    <section class="preview-section" aria-labelledby="preview-title"><div class="section-label"><span>LIVE READOUT</span><h2 id="preview-title">See what still needs attention</h2></div>${previewPanel()}</section>
    <section class="how-section" aria-labelledby="how-title"><div class="section-label"><span>OPERATING SEQUENCE</span><h2 id="how-title">How it works</h2></div><ol class="steps"><li><b>1</b><div><h3>Add records</h3><p>Enter a payment, attach a receipt, or import a bank CSV.</p></div></li><li><b>2</b><div><h3>Check the quarter</h3><p>Resolve each category, then confirm the figures yourself.</p></div></li><li><b>3</b><div><h3>Hand it over</h3><p>Download a CSV or make a 30-day accountant link.</p></div></li></ol></section>
    <section class="limits-section" aria-labelledby="limits-title"><div><p class="eyebrow">CLEAR LIMITS</p><h2 id="limits-title">Records and checks, not tax advice</h2><p>Quarterly Ready does not run payroll or decide what you can claim. It does not send data straight to HMRC.</p><p>You review every figure. Then use the handoff with your accountant or HMRC-recognised software.</p></div><div class="privacy-plate"><span class="plate-title">DATA POSITION</span><p>Real records use encrypted server storage. Demo data stays in a separate browser key.</p><a href="/privacy" data-link>Read the privacy notice</a></div></section>
    ${pricingSection()}
  </main>`);
}

function previewPanel(): string {
  return `<div class="mini-panel" aria-label="Sample quarter preview"><div class="mini-summary"><div><span>INCOME</span><strong>£260.00</strong></div><div><span>COSTS</span><strong>£155.83</strong></div><div><span>NET</span><strong>£104.17</strong></div></div><div class="preview-track"><span class="track-fill"></span></div><div class="preview-line"><span class="lamp orange"></span><b>1 transaction needs a category</b><a href="/demo" data-link>Open sample quarter</a></div></div>`;
}

function pricingSection(): string {
  return `<section class="pricing-section" aria-labelledby="price-title"><div><p class="eyebrow">FOUNDER LICENCE</p><h2 id="price-title">Share live packs for £99</h2><p>One payment adds renewable 30-day accountant links on this browser.</p><p>The free version keeps your quarter and every download.</p></div><div class="price-control"><strong><span>£</span>99</strong><span>one-time purchase</span><a class="primary-button" href="${BILLING}/checkout">Buy the full version</a><button class="text-button" id="show-license">Have a licence? Paste it</button><form id="license-form" class="license-form" hidden><label for="license-token">Licence token</label><div><input id="license-token" name="license" autocomplete="off" required><button type="submit">Verify licence</button></div></form><p id="license-result" class="form-message" aria-live="polite"></p><small>Sociobot is the merchant of record. Refunds are handled there.</small></div></section>`;
}

function recordsPage(demo: boolean): string {
  currentDemo = demo;
  currentDocument = loadDocument(demo);
  const doc = currentDocument;
  const sum = summarise(doc);
  const completion = checklist(doc).filter(item => item.done).length;
  return layout(`<main id="main" class="app-main">
    <div class="app-heading"><div><p class="eyebrow">QUARTER 1 · 2026–27</p><h1 tabindex="-1">Check this quarter</h1><p>${escapeHtml(doc.quarterLabel)} · ${escapeHtml(doc.businessName || 'Business name not entered')}</p></div><div class="connection" role="status"><span class="lamp ${navigator.onLine ? 'teal' : 'orange'}"></span>${navigator.onLine ? (demo ? 'Demo ready' : 'Saved in this browser') : 'Offline — browser copy active'}</div></div>
    <details class="business-settings"><summary>Business details</summary><form id="business-form"><label for="business-name">Business name</label><div><input id="business-name" name="businessName" maxlength="100" value="${escapeHtml(doc.businessName)}" required><button type="submit">Save business name</button></div></form></details>
    ${notice ? `<div class="notice" role="status">${escapeHtml(notice)}</div>` : ''}
    <section class="control-panel" aria-labelledby="summary-title">
      <div class="dial-block"><div class="quarter-dial dial-${completion}" aria-label="${completion} of 4 checks complete"><span class="dial-hand"></span><span class="dial-centre"></span><i>1</i><i>2</i><i>3</i><i>4</i></div><div><span class="control-label">READINESS</span><strong>${completion} / 4 checks</strong></div></div>
      <div class="readout"><h2 id="summary-title">Quarter totals</h2><dl><div><dt>Income</dt><dd>${pounds(sum.incomePence)}</dd></div><div><dt>Costs</dt><dd>${pounds(sum.expensePence)}</dd></div><div class="net"><dt>Net</dt><dd>${pounds(sum.netPence)}</dd></div></dl></div>
      <div class="signal-box"><span class="lamp ${sum.unresolved ? 'orange' : 'teal'}"></span><div><strong>${sum.unresolved ? `${sum.unresolved} ${sum.unresolved === 1 ? 'transaction needs' : 'transactions need'} a category` : 'Every transaction has a category'}</strong><span>${sum.missingReceipts} expense ${sum.missingReceipts === 1 ? 'receipt is' : 'receipts are'} missing</span></div></div>
    </section>
    <section class="records-section" aria-labelledby="records-title"><div class="section-toolbar"><div><p class="eyebrow">DIGITAL RECORDS</p><h2 id="records-title">Transactions</h2></div><div><button id="toggle-add" class="primary-button">Add a transaction</button><label class="file-button">Import bank CSV<input id="csv-input" type="file" accept=".csv,text/csv"></label></div></div>
      <form id="add-form" class="paper-form" hidden><h3>Add one transaction</h3><div class="form-grid"><label>Date<input name="date" type="date" min="${doc.quarterStart}" max="${doc.quarterEnd}" required></label><label>Description<input name="description" maxlength="120" required></label><label>Amount in pounds<input name="amount" inputmode="decimal" pattern="[0-9]+([.][0-9]{1,2})?" required></label><label>Type<select name="kind"><option value="income">Income</option><option value="expense">Expense</option></select></label><label>Category<select name="category">${categoryOptions('')}</select></label><label>Receipt, if you have one<input name="receipt" type="file" accept="image/jpeg,image/png,application/pdf"></label></div><div class="form-actions"><button type="submit">Save transaction</button><button type="button" class="text-button" id="cancel-add">Cancel</button></div><p class="form-message" id="add-error" aria-live="polite"></p></form>
      ${transactionTable(doc.transactions)}
    </section>
    <section class="review-section" aria-labelledby="review-title"><div class="section-label"><span>HUMAN REVIEW</span><h2 id="review-title">Quarter checklist</h2></div><ol class="checklist">${checklist(doc).map((item, index) => `<li class="${item.done ? 'done' : ''}"><span>${item.done ? '✓' : index + 1}</span><div><strong>${item.title}</strong><small>${item.detail}</small></div>${item.control || ''}</li>`).join('')}</ol><div class="ready-control"><button id="mark-ready" ${completion < 4 ? 'disabled' : ''}>Mark quarter ready</button><p>${doc.markedReady ? '<strong>Quarter marked ready.</strong> Export the reviewed handoff when you choose.' : 'Complete all four checks before marking the quarter ready.'}</p></div></section>
    <section class="handoff-section" aria-labelledby="handoff-title"><div><p class="eyebrow">OUTPUT BAY</p><h2 id="handoff-title">Prepare the handoff</h2><p>Downloads stay available in the free version.</p></div><div class="output-controls"><button id="download-pack">Download accountant CSV</button><button id="share-pack" ${!demo && !isLicensed() ? 'aria-describedby="share-note"' : ''}>Make accountant link</button><button id="download-hmrc" ${sum.unresolved || !doc.figuresReviewed ? 'disabled' : ''}>Download HMRC handoff</button><p id="share-note">${!demo && !isLicensed() ? 'Live accountant links need the £99 licence. The demo link still works.' : 'Accountant links expire after 30 days.'}</p><p id="output-result" class="form-message" aria-live="polite"></p></div></section>
    <aside class="submission-note"><strong>No direct HMRC submission</strong><p>HMRC must recognise production software before it can submit. This version creates a reviewed handoff for recognised software.</p></aside>
  </main>`, demo);
}

function transactionTable(transactions: Transaction[]): string {
  if (!transactions.length) return `<div class="empty-state"><span class="empty-dial" aria-hidden="true"></span><h3>No transactions in this quarter</h3><p>Add a payment or import a bank CSV. Your quarter totals will appear here.</p><button id="empty-add">Add the first transaction</button></div>`;
  return `<div class="table-wrap"><table><caption class="sr-only">Transactions for this quarter</caption><thead><tr><th scope="col">Date</th><th scope="col">Description</th><th scope="col">Type</th><th scope="col">Amount</th><th scope="col">Category</th><th scope="col"><span class="sr-only">Actions</span></th></tr></thead><tbody>${transactions.map(t => `<tr data-id="${t.id}"><td data-label="Date">${formatDate(t.date)}</td><td data-label="Description"><strong>${escapeHtml(t.description)}</strong>${t.receiptName ? `<span class="receipt-mark">Receipt · ${escapeHtml(t.receiptName)}</span>` : ''}</td><td data-label="Type"><span class="kind ${t.kind}">${t.kind}</span></td><td data-label="Amount" class="money">${pounds(t.amountPence)}</td><td data-label="Category"><label class="sr-only" for="category-${t.id}">Category for ${escapeHtml(t.description)}</label><select id="category-${t.id}" class="category-select ${t.category ? '' : 'unresolved'}" data-category>${categoryOptions(t.category)}</select></td><td>${t.kind === 'expense' && !t.receiptName ? `<label class="attach-button">Attach receipt<input data-receipt type="file" accept="image/jpeg,image/png,application/pdf"><span class="sr-only"> for ${escapeHtml(t.description)}</span></label>` : ''}<button class="delete-button" data-delete aria-label="Delete ${escapeHtml(t.description)}">Delete</button></td></tr>`).join('')}</tbody></table></div>`;
}

function categoryOptions(selected: Category): string {
  return CATEGORIES.map(category => `<option value="${escapeHtml(category)}" ${category === selected ? 'selected' : ''}>${category || 'Needs a category'}</option>`).join('');
}

function checklist(doc: QuarterDocument): { title: string; detail: string; done: boolean; control?: string }[] {
  const sum = summarise(doc);
  return [
    { title: 'Every transaction has a category', detail: sum.unresolved ? `${sum.unresolved} still need attention.` : 'Categories are complete.', done: sum.unresolved === 0 },
    { title: 'Expense receipts checked', detail: sum.missingReceipts ? `${sum.missingReceipts} expenses have no receipt attached.` : 'Every expense has a receipt.', done: sum.missingReceipts === 0 },
    { title: 'Figures reviewed by you', detail: 'Confirm the totals against your source records.', done: doc.figuresReviewed, control: `<label class="switch-label"><input id="figures-reviewed" type="checkbox" ${doc.figuresReviewed ? 'checked' : ''}>I checked these figures</label>` },
    { title: 'Accountant pack downloaded', detail: doc.packDownloaded ? 'A CSV was prepared in this browser.' : 'Download the CSV before handoff.', done: doc.packDownloaded }
  ];
}

function privacyPage(): string {
  return layout(`<main id="main" class="prose-page"><p class="eyebrow">LEGAL · 28 AUGUST 2026</p><h1 tabindex="-1">Privacy in plain words</h1><p class="lede">Quarterly Ready stores the records you enter so you can return to your quarter.</p><h2>What we store</h2><p>We store your transaction document under a random browser workspace ID. The server encrypts that document before writing it to SQLite.</p><p>Receipt file contents stay in the document when you attach them. Accountant links store an encrypted snapshot for 30 days.</p><h2>Demo data</h2><p>The demo uses sample records in a separate browser storage key. It does not read, write, or copy your real records.</p><h2>Payments</h2><p>Sociobot handles checkout and licence checks. Dodo is its payment provider. Quarterly Ready stores the licence token in your browser.</p><h2>What we do not collect</h2><p>There are no advertising cookies or third-party analytics. The server keeps only a daily page count without an IP address.</p><h2>Your choices</h2><p>Delete your browser data to remove its local copy. Email <a href="mailto:privacy@sociobot.in">privacy@sociobot.in</a> to request deletion of server records.</p></main>`);
}

function termsPage(): string {
  return layout(`<main id="main" class="prose-page"><p class="eyebrow">LEGAL · 28 AUGUST 2026</p><h1 tabindex="-1">Terms for using Quarterly Ready</h1><p class="lede">These terms cover the records tool and the £99 one-time licence.</p><h2>Use of the service</h2><p>You may use Quarterly Ready for lawful UK business records. Keep your own backups of important exports.</p><h2>No tax advice</h2><p>The tool organises figures but does not decide tax treatment. You remain responsible for checking records and meeting deadlines.</p><h2>HMRC handoff</h2><p>The handoff file is not a submission. Review it in HMRC-recognised software or with an accountant before sending anything.</p><h2>Paid licence</h2><p>The £99 licence is a one-time purchase for the current major version. Sociobot is the merchant of record and handles refunds.</p><h2>Availability</h2><p>We aim to keep the service available but cannot promise uninterrupted access. The free CSV export helps you keep a portable copy.</p><h2>Contact</h2><p>Email <a href="mailto:support@sociobot.in">support@sociobot.in</a> with service or licence questions.</p></main>`);
}

function sharePage(token: string): string {
  return layout(`<main id="main" class="prose-page"><p class="eyebrow">READ-ONLY ACCOUNTANT LINK</p><h1 tabindex="-1">Review this accountant pack</h1><div id="shared-pack" class="loading-state" role="status"><span class="spinner" aria-hidden="true"></span><p>Opening the encrypted snapshot…</p></div></main>`);
}

function notFoundPage(): string {
  return layout(`<main id="main" class="not-found"><div class="lost-dial" aria-hidden="true"><span></span></div><p class="eyebrow">NO SIGNAL</p><h1 tabindex="-1">This page is not on the panel</h1><p>The address may have changed.</p><a class="primary-button" href="/" data-link>Return home</a></main>`);
}

function route(): void {
  const path = location.pathname.replace(/\/+$/, '') || '/';
  notice = '';
  if (path === '/') { setPage('Quarterly Ready — Check your MTD quarter', homePage()); bindHome(); }
  else if (path === '/demo') { setPage('Demo — Quarterly Ready', recordsPage(true)); bindRecords(); }
  else if (path === '/records') { setPage('Records — Quarterly Ready', recordsPage(false)); bindRecords(); void refreshRemote(); }
  else if (path === '/privacy') setPage('Privacy — Quarterly Ready', privacyPage());
  else if (path === '/terms') setPage('Terms — Quarterly Ready', termsPage());
  else if (path.startsWith('/share/')) { setPage('Accountant pack — Quarterly Ready', sharePage(path.slice(7))); void renderShare(path.slice(7)); }
  else { setPage('Page not found — Quarterly Ready', notFoundPage()); }
  bindLinks();
}

function setPage(title: string, html: string): void {
  document.title = title;
  app.innerHTML = html;
  const heading = app.querySelector<HTMLHeadingElement>('h1');
  document.querySelector<HTMLDivElement>('#route-status')!.textContent = heading?.textContent || title;
  requestAnimationFrame(() => heading?.focus({ preventScroll: true }));
  scrollTo({ top: 0, behavior: matchMedia('(prefers-reduced-motion: reduce)').matches ? 'auto' : 'smooth' });
}

function bindLinks(): void {
  app.querySelectorAll<HTMLAnchorElement>('a[data-start-real]').forEach(link => link.addEventListener('click', () => leaveDemo(), { once: true }));
}

function bindHome(): void {
  document.querySelector('#show-license')?.addEventListener('click', () => { const form = document.querySelector<HTMLFormElement>('#license-form')!; form.hidden = false; form.querySelector('input')?.focus(); });
  document.querySelector('#license-form')?.addEventListener('submit', async event => {
    event.preventDefault(); const form = event.currentTarget as HTMLFormElement; const token = new FormData(form).get('license')?.toString().trim(); if (token) await storeAndVerifyLicense(token);
  });
  if (!sessionStorage.getItem('quarterly-ready:viewed')) {
    sessionStorage.setItem('quarterly-ready:viewed', '1'); void fetch('/api/page-view', { method: 'POST' }).catch(() => undefined);
  }
}

function bindRecords(): void {
  document.querySelector('#reset-demo')?.addEventListener('click', () => { currentDocument = resetDemo(); notice = 'The sample quarter was reset.'; rerenderRecords(); });
  const showForm = () => { const form = document.querySelector<HTMLFormElement>('#add-form')!; form.hidden = false; form.querySelector<HTMLInputElement>('input')?.focus(); };
  document.querySelector('#toggle-add')?.addEventListener('click', showForm);
  document.querySelector('#empty-add')?.addEventListener('click', showForm);
  document.querySelector('#cancel-add')?.addEventListener('click', () => { document.querySelector<HTMLFormElement>('#add-form')!.hidden = true; });
  document.querySelector('#add-form')?.addEventListener('submit', addTransaction);
  document.querySelector('#business-form')?.addEventListener('submit', event => { event.preventDefault(); const value = new FormData(event.currentTarget as HTMLFormElement).get('businessName')?.toString().trim(); if (value) { currentDocument!.businessName = value; saveAndRender('Business name saved.'); } });
  document.querySelector<HTMLInputElement>('#csv-input')?.addEventListener('change', importCsv);
  document.querySelectorAll<HTMLSelectElement>('[data-category]').forEach(select => select.addEventListener('change', () => {
    const row = select.closest<HTMLTableRowElement>('tr')!; const transaction = currentDocument!.transactions.find(t => t.id === row.dataset.id)!;
    transaction.category = select.value as Category; saveAndRender('Category saved.');
  }));
  document.querySelectorAll<HTMLButtonElement>('[data-delete]').forEach(button => button.addEventListener('click', () => {
    const row = button.closest<HTMLTableRowElement>('tr')!; const transaction = currentDocument!.transactions.find(t => t.id === row.dataset.id)!;
    if (confirm(`Delete “${transaction.description}”? This cannot be undone.`)) { currentDocument!.transactions = currentDocument!.transactions.filter(t => t.id !== transaction.id); saveAndRender('Transaction deleted.'); }
  }));
  document.querySelectorAll<HTMLInputElement>('[data-receipt]').forEach(input => input.addEventListener('change', async () => {
    const file = input.files?.[0]; if (!file) return;
    if (file.size > 1_500_000) { notice = 'The receipt is larger than 1.5 MB. Choose a smaller image or PDF.'; rerenderRecords(); return; }
    const row = input.closest<HTMLTableRowElement>('tr')!; const transaction = currentDocument!.transactions.find(t => t.id === row.dataset.id)!;
    transaction.receiptName = file.name; transaction.receiptData = await fileToDataUrl(file); saveAndRender('Receipt attached.');
  }));
  document.querySelector<HTMLInputElement>('#figures-reviewed')?.addEventListener('change', event => { currentDocument!.figuresReviewed = (event.target as HTMLInputElement).checked; saveAndRender('Review choice saved.'); });
  document.querySelector('#download-pack')?.addEventListener('click', downloadPack);
  document.querySelector('#download-hmrc')?.addEventListener('click', downloadHmrc);
  document.querySelector('#share-pack')?.addEventListener('click', sharePack);
  document.querySelector('#mark-ready')?.addEventListener('click', () => { currentDocument!.markedReady = true; saveAndRender('Quarter marked ready.'); });
}

async function addTransaction(event: Event): Promise<void> {
  event.preventDefault();
  const form = event.currentTarget as HTMLFormElement; const data = new FormData(form); const file = data.get('receipt') as File;
  const amount = Number(data.get('amount'));
  const message = form.querySelector<HTMLParagraphElement>('#add-error')!;
  if (!Number.isFinite(amount) || amount <= 0) { message.textContent = 'The amount must be more than zero. Enter pounds and pence.'; return; }
  if (file?.size > 1_500_000) { message.textContent = 'The receipt is larger than 1.5 MB. Choose a smaller image or PDF.'; return; }
  let receiptData = '';
  if (file?.size) receiptData = await fileToDataUrl(file);
  currentDocument!.transactions.push({ id: crypto.randomUUID(), date: String(data.get('date')), description: String(data.get('description')).trim(), amountPence: Math.round(amount * 100), kind: data.get('kind') as Transaction['kind'], category: data.get('category') as Category, receiptName: file?.size ? file.name : undefined, receiptData: receiptData || undefined });
  saveAndRender('Transaction added.');
}

async function importCsv(event: Event): Promise<void> {
  const input = event.target as HTMLInputElement; const file = input.files?.[0]; if (!file) return;
  try {
    const imported = parseCsv(await file.text()).map(row => ({ ...row, id: crypto.randomUUID() }));
    currentDocument!.transactions.push(...imported); saveAndRender(`${imported.length} transactions imported.`);
  } catch (error) { notice = error instanceof Error ? error.message : 'The CSV could not be imported. Check the columns and try again.'; rerenderRecords(); }
}

function downloadPack(): void {
  downloadBlob(`accountant-pack-${currentDocument!.quarterStart}.csv`, accountantCsv(currentDocument!), 'text/csv;charset=utf-8');
  currentDocument!.packDownloaded = true; saveAndRender('Accountant CSV downloaded.');
}

function downloadHmrc(): void {
  downloadBlob(`hmrc-handoff-${currentDocument!.quarterStart}.json`, JSON.stringify(hmrcHandoff(currentDocument!), null, 2), 'application/json');
  const result = document.querySelector('#output-result'); if (result) result.textContent = 'HMRC handoff downloaded. Review it in recognised software.';
}

async function sharePack(): Promise<void> {
  const result = document.querySelector<HTMLParagraphElement>('#output-result')!;
  if (!currentDemo && !isLicensed()) { result.textContent = 'A live accountant link needs the £99 licence. The CSV remains free.'; return; }
  try {
    const url = currentDemo ? `${location.origin}/share/demo` : await createShare(currentDocument!);
    await navigator.clipboard.writeText(url).catch(() => undefined);
    result.innerHTML = `Accountant link ready and copied: <a href="${url}" data-link>${url}</a>`; bindLinks();
  } catch (error) { result.textContent = error instanceof Error ? error.message : 'The accountant link was not created. Try again.'; }
}

function saveAndRender(message: string): void { saveDocument(currentDocument!, currentDemo); notice = message; rerenderRecords(); }
function rerenderRecords(): void { setPage(currentDemo ? 'Demo — Quarterly Ready' : 'Records — Quarterly Ready', recordsPage(currentDemo)); bindRecords(); bindLinks(); }

async function refreshRemote(): Promise<void> {
  if (!navigator.onLine || currentDocument!.transactions.length) return;
  try { const remote = await loadRemote(); if (remote) { currentDocument = remote; notice = 'Saved records loaded.'; rerenderRecords(); } }
  catch (error) { notice = error instanceof Error ? error.message : 'Saved records could not be loaded.'; rerenderRecords(); }
}

async function renderShare(token: string): Promise<void> {
  const target = document.querySelector<HTMLDivElement>('#shared-pack')!;
  try {
    const doc = await loadShare(token); const sum = summarise(doc);
    target.className = 'shared-document';
    target.innerHTML = `<div class="mini-summary"><div><span>INCOME</span><strong>${pounds(sum.incomePence)}</strong></div><div><span>COSTS</span><strong>${pounds(sum.expensePence)}</strong></div><div><span>NET</span><strong>${pounds(sum.netPence)}</strong></div></div><h2>${escapeHtml(doc.businessName || 'Unnamed business')}</h2><p>${escapeHtml(doc.quarterLabel)} · ${doc.transactions.length} transactions · ${sum.unresolved} unresolved</p>${transactionTable(doc.transactions).replaceAll(/<select[\s\S]*?<\/select>/g, '<span>Read only</span>').replaceAll(/<button[\s\S]*?<\/button>/g, '')}`;
  } catch (error) { target.className = 'error-state'; target.innerHTML = `<h2>The pack did not open</h2><p>${escapeHtml(error instanceof Error ? error.message : 'Ask the owner for a new accountant link.')}</p>`; }
}

function downloadBlob(name: string, contents: string, type: string): void {
  const url = URL.createObjectURL(new Blob([contents], { type })); const link = document.createElement('a');
  link.href = url; link.download = name; link.click(); setTimeout(() => URL.revokeObjectURL(url), 1000);
}

function fileToDataUrl(file: File): Promise<string> { return new Promise((resolve, reject) => { const reader = new FileReader(); reader.onload = () => resolve(String(reader.result)); reader.onerror = () => reject(new Error('The receipt could not be read. Choose it again.')); reader.readAsDataURL(file); }); }
function formatDate(value: string): string { return new Intl.DateTimeFormat('en-GB', { day: 'numeric', month: 'short' }).format(new Date(`${value}T12:00:00Z`)); }

const LICENSE_KEY = `sb_license:${SLUG}`;
const VERDICT_KEY = `sb_license_verdict:${SLUG}`;
function isLicensed(): boolean { try { const result = JSON.parse(localStorage.getItem(VERDICT_KEY) || '{}'); return result.valid === true; } catch { return false; } }
async function storeAndVerifyLicense(token: string): Promise<void> {
  localStorage.setItem(LICENSE_KEY, token); const output = document.querySelector<HTMLParagraphElement>('#license-result');
  if (output) output.textContent = 'Checking the licence…';
  try { const response = await fetch(`${BILLING}/verify?license=${encodeURIComponent(token)}`); const result = await response.json() as { valid: boolean; reason: string; expires_at?: string }; localStorage.setItem(VERDICT_KEY, JSON.stringify({ ...result, checkedAt: Date.now() })); if (output) output.textContent = result.valid ? 'Licence active on this browser.' : 'This licence is not active. Check the token or buy a new licence.'; }
  catch { if (output) output.textContent = 'The licence service could not be reached. Check your connection and try again.'; }
}

function handleLicenseReturn(): void {
  const params = new URLSearchParams(location.search); const token = params.get('license');
  if (token) { localStorage.setItem(LICENSE_KEY, token); params.delete('license'); history.replaceState({}, '', `${location.pathname}${params.size ? `?${params}` : ''}`); void storeAndVerifyLicense(token); }
  const saved = localStorage.getItem(LICENSE_KEY); if (!saved) return;
  try { const verdict = JSON.parse(localStorage.getItem(VERDICT_KEY) || '{}'); if (!verdict.checkedAt || Date.now() - verdict.checkedAt > 86_400_000) void storeAndVerifyLicense(saved); } catch { void storeAndVerifyLicense(saved); }
}

document.addEventListener('click', event => { const link = (event.target as HTMLElement).closest<HTMLAnchorElement>('a[data-link]'); if (link && link.origin === location.origin) { event.preventDefault(); history.pushState({}, '', link.pathname); route(); } });
addEventListener('popstate', route);
addEventListener('online', () => { if (location.pathname === '/records' || location.pathname === '/demo') rerenderRecords(); });
addEventListener('offline', () => { if (location.pathname === '/records' || location.pathname === '/demo') rerenderRecords(); });
addEventListener('save-error', () => { if (!currentDemo) { notice = 'The server copy was not saved. Your browser copy is still available.'; rerenderRecords(); } });

handleLicenseReturn();
route();
if ('serviceWorker' in navigator && import.meta.env.PROD) addEventListener('load', () => void navigator.serviceWorker.register('/sw.js'));
