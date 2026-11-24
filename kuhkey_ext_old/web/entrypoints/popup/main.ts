import './style.css';
import cowurl from '../../assets/cow.jpeg';
import { mockEntries, type Entry } from '../../components/test';
import initWasm, { generate_password } from '../../wasm/kuhkey_ext.js';
import { browser } from 'wxt/browser';

// Load the Rust/WASM module once, then rust functions can only be called after this finishes
const wasmReady = initWasm();

const MASK = '🐄🐮🐄🐮🐄🐮🐄🐮';

const list = document.querySelector<HTMLElement>('#entries')!;
const detail = document.querySelector<HTMLElement>('#detail')!;
const notify = document.querySelector<HTMLElement>('.copy-notify')!;
const notifyText = notify.querySelector<HTMLElement>('h2')!;
const addButton = document.querySelector<HTMLButtonElement>('#add-entry')!;

renderList(mockEntries);
showCow();
addButton.addEventListener('click', () => showEntryForm());

// login screen --> any user + any psw works
const login = document.querySelector<HTMLFormElement>('#login')!;
login.querySelector<HTMLImageElement>('.login-cow')!.src = cowurl;
login.addEventListener('submit', (e) => {
  e.preventDefault(); // don't reload the page
  login.remove();
  document.querySelector<HTMLElement>('.title-box')!.hidden = false;
  document.querySelector<HTMLElement>('.layout')!.hidden = false;

  // empty username -> just "welcome back!"
  const user = login.querySelector<HTMLInputElement>('#login-username')!.value.trim();
  showCopyNotify(user ? `Welcome back ${user}!` : 'Welcome back!');
});

// draws the list, the selected entry gets highlighted + scrolled to
function renderList(entries: Entry[], selected?: Entry) {
  list.replaceChildren();
  let selectedItem: HTMLElement | undefined;

  for (const entry of entries) {
    const item = document.createElement('button');
    item.className = 'entry-item';

    const logo = entryLogo(entry);

    const info = document.createElement('div');
    info.className = 'entry-info';

    const title = document.createElement('div');
    title.className = 'entry-title';
    title.textContent = entry.title;

    const username = document.createElement('div');
    username.className = 'entry-username';
    username.textContent = entry.username;

    info.append(title, username);
    item.append(logo, info);

    item.addEventListener('click', () => {
      list.querySelector('.entry-item.selected')?.classList.remove('selected');
      item.classList.add('selected');
      showEntry(entry);
    });

    if (entry === selected) {
      item.classList.add('selected');
      selectedItem = item;
    }

    list.append(item);
  }

  selectedItem?.scrollIntoView({ block: 'nearest' });
}

function showCow() {
  const img = document.createElement('img');
  img.className = 'detail-cow';
  img.src = cowurl;
  img.alt = 'KuhKey cow';
  detail.replaceChildren(img);
}

function showEntry(entry: Entry) {
  const heading = document.createElement('h2');
  heading.className = 'detail-title';
  heading.textContent = entry.title;

  const close = document.createElement('button');
  close.className = 'detail-close';
  close.textContent = '✕';
  close.title = 'Close';
  close.setAttribute('aria-label', 'Close');
  close.addEventListener('click', () => {
    list.querySelector('.entry-item.selected')?.classList.remove('selected');
    showCow();
  });

  // edit the whole entry (name, website, username, psw)
  const editEntry = smallButton('Edit', 'Edit entry');
  editEntry.addEventListener('click', () => showEntryForm(entry));

  const headerButtons = document.createElement('div');
  headerButtons.className = 'detail-actions';
  headerButtons.append(editEntry, close);

  const header = document.createElement('div');
  header.className = 'detail-header';
  header.append(heading, headerButtons);

  const card = document.createElement('div');
  card.className = 'field-card';
  card.append(
    usernameField(entry.username),
    passwordField(entry),
  );
  if (entry.url) {
    card.append(websiteField(entry.url));
  }

  const content = document.createElement('div');
  content.className = 'detail-content';
  content.append(header, card);

  detail.replaceChildren(content);
}

function field(label: string, value: string): HTMLElement {
  const row = document.createElement('div');
  row.className = 'field';

  const labelEl = document.createElement('div');
  labelEl.className = 'field-label';
  labelEl.textContent = label;

  const valueEl = document.createElement('div');
  valueEl.className = 'field-value';
  valueEl.textContent = value;

  row.append(labelEl, valueEl);
  return row;
}

let notifyTimer: number | undefined;

function showCopyNotify(text: string) {
  notifyText.textContent = text;
  notify.classList.remove('show');
  void notify.offsetWidth;
  notify.classList.add('show');

  clearTimeout(notifyTimer);
  notifyTimer = window.setTimeout(() => notify.classList.remove('show'), 1000);
}

// psw gen setting

interface GeneratorOptions {
  length: number;
  uppercase: boolean;
  lowercase: boolean;
  numbers: boolean;
  symbols: boolean;
  excludeLookalikes: boolean;
}

type CharacterType = 'uppercase' | 'lowercase' | 'numbers' | 'symbols';

// Starts like PasswordOptions::default() in Rust. Changes are kept while the popup is open,
// so the next "Generate" starts with your last settings.
const genOptions: GeneratorOptions = {
  length: 20,
  uppercase: true,
  lowercase: true,
  numbers: true,
  symbols: true,
  excludeLookalikes: false,
};

const MIN_LENGTH = 4; // always room for one character of each type
const MAX_LENGTH = 64;

/** Asks Rust (kuhkey_core) for a new password with the current settings. */
function newPassword(): string {
  return generate_password(
    genOptions.length,
    genOptions.uppercase,
    genOptions.lowercase,
    genOptions.numbers,
    genOptions.symbols,
    genOptions.excludeLookalikes,
  );
}


function smallButton(text: string, label: string): HTMLButtonElement {
  const button = document.createElement('button');
  button.type = 'button';
  button.className = 'field-edit';
  button.textContent = text;
  button.title = label;
  button.setAttribute('aria-label', label);
  return button;
}

function toggle(text: string, title: string, checked: boolean) {
  const checkbox = document.createElement('input');
  checkbox.type = 'checkbox';
  checkbox.checked = checked;
  checkbox.setAttribute('aria-label', title);

  const label = document.createElement('label');
  label.className = 'gen-toggle';
  label.title = title;
  label.append(checkbox, text);

  return { label, checkbox };
}

// Length slider + toggles for the char types
function generatorSettings(onChange: () => void): HTMLElement {
  // lenght
  const lengthLabel = document.createElement('span');
  lengthLabel.className = 'gen-label';
  lengthLabel.textContent = 'Length';

  const slider = document.createElement('input');
  slider.type = 'range';
  slider.className = 'gen-slider';
  slider.min = String(MIN_LENGTH);
  slider.max = String(MAX_LENGTH);
  slider.value = String(genOptions.length);
  slider.setAttribute('aria-label', 'Password length');

  const lengthValue = document.createElement('span');
  lengthValue.className = 'gen-length-value';
  lengthValue.textContent = String(genOptions.length);

  slider.addEventListener('input', () => {
    genOptions.length = Number(slider.value);
    lengthValue.textContent = slider.value;
    onChange();
  });

  const lengthRow = document.createElement('div');
  lengthRow.className = 'gen-length';
  lengthRow.append(lengthLabel, slider, lengthValue);

  // char types
  const characterTypes: { key: CharacterType; text: string; title: string }[] = [
    { key: 'uppercase', text: 'A-Z', title: 'Uppercase letters' },
    { key: 'lowercase', text: 'a-z', title: 'Lowercase letters' },
    { key: 'numbers', text: '0-9', title: 'Numbers' },
    { key: 'symbols', text: '!@#', title: 'Symbols' },
  ];

  const typeRow = document.createElement('div');
  typeRow.className = 'gen-types';
  const typeBoxes: HTMLInputElement[] = [];

  // very important ! at least one box has to be ticked,  (we can generate psw with no chars)
  const lockLastType = () => {
    const onCount = typeBoxes.filter((box) => box.checked).length;
    for (const box of typeBoxes) {
      box.disabled = box.checked && onCount === 1;
    }
  };

  for (const type of characterTypes) {
    const { label, checkbox } = toggle(type.text, type.title, genOptions[type.key]);
    checkbox.addEventListener('change', () => {
      genOptions[type.key] = checkbox.checked;
      lockLastType();
      onChange();
    });
    typeBoxes.push(checkbox);
    typeRow.append(label);
  }
  lockLastType();

  const lookalikes = toggle(
    'Avoid look-alikes (0 O o 1 I l)',
    'Leave out characters that are easy to mix up',
    genOptions.excludeLookalikes,
  );
  lookalikes.checkbox.addEventListener('change', () => {
    genOptions.excludeLookalikes = lookalikes.checkbox.checked;
    onChange();
  });

  const settings = document.createElement('div');
  settings.className = 'gen-settings';
  settings.append(lengthRow, typeRow, lookalikes.label);
  return settings;
}


let clearTimer: number | undefined;

function passwordField(entry: Entry): HTMLElement {
  const row = field('Password', MASK);
  const valueEl = row.querySelector<HTMLElement>('.field-value')!;
  valueEl.classList.add('field-password');
  valueEl.tabIndex = 0; // so it can be reached with tab

  // so it only reveals the password when a user intends to do it
  const reveal = () => (valueEl.textContent = entry.password);
  const hide = () => (valueEl.textContent = MASK);
  valueEl.addEventListener('mouseenter', reveal);
  valueEl.addEventListener('mouseleave', hide);
  valueEl.addEventListener('focus', reveal);
  valueEl.addEventListener('blur', hide);

  // on click, copy it and clear the clipboard again after 30 sec
  valueEl.addEventListener('click', () => {
    navigator.clipboard.writeText(entry.password);
    clearTimeout(clearTimer);
    clearTimer = window.setTimeout(() => navigator.clipboard.writeText(''), 30_000);
    showCopyNotify('Password copied!');
  });

  return row;
}

function usernameField(username: string): HTMLElement {
  const row = field('Username', username);
  const valueEl = row.querySelector<HTMLElement>('.field-value')!;
  valueEl.classList.add('field-username');
  valueEl.tabIndex = 0;

  valueEl.addEventListener('click', () => {
    navigator.clipboard.writeText(username);
    showCopyNotify('Username copied!');
  });

  return row;
}

// website as a link, opens in a new tab
function websiteField(url: string): HTMLElement {
  const row = field('Website', '');
  const valueEl = row.querySelector<HTMLElement>('.field-value')!;

  const link = document.createElement('a');
  link.className = 'field-link';
  link.href = url; // always http(s), checked in normalizeUrl()
  link.target = '_blank';
  link.rel = 'noopener noreferrer';
  link.textContent = new URL(url).hostname;

  valueEl.append(link);
  return row;
}

// add new psw

function textInput(placeholder: string, label: string): HTMLInputElement {
  const input = document.createElement('input');
  input.className = 'field-input';
  input.type = 'text';
  input.placeholder = placeholder;
  input.spellcheck = false;
  input.autocomplete = 'off';
  input.setAttribute('aria-label', label);
  // red border goes away when you type
  input.addEventListener('input', () => input.classList.remove('invalid'));
  return input;
}

function formField(label: string, ...content: HTMLElement[]): HTMLElement {
  const row = document.createElement('div');
  row.className = 'field';

  const labelEl = document.createElement('div');
  labelEl.className = 'field-label';
  labelEl.textContent = label;

  row.append(labelEl, ...content);
  return row;
}

// "github.com" -> "https://github.com/", returns null if its not http(s) (so no javascript: links)
function normalizeUrl(text: string): string | null {
  const trimmed = text.trim();
  const withScheme = /^[a-z][a-z0-9+.-]*:/i.test(trimmed) ? trimmed : `https://${trimmed}`;
  try {
    const url = new URL(withScheme);
    if (url.protocol !== 'http:' && url.protocol !== 'https:') return null;
    return url.href;
  } catch {
    return null;
  }
}

// gets the site of the current tab, e.g. github.com
async function currentSite(): Promise<{ name: string; url: string } | null> {
  try {
    const [tab] = await browser.tabs.query({ active: true, currentWindow: true });
    if (!tab?.url) return null;

    const url = new URL(tab.url);
    if (url.protocol !== 'http:' && url.protocol !== 'https:') return null; // e.g. chrome://newtab

    return { name: siteName(url.hostname), url: url.origin };
  } catch {
    return null; // not running as extension or no permission
  }
}

// domain -> name, e.g. "www.youtube.com" -> "Youtube", "bbc.co.uk" -> "Bbc"
function siteName(hostname: string): string {
  // ip or localhost etc, just keep it like it is
  if (/^[\d.]+$/.test(hostname) || hostname.includes(':') || !hostname.includes('.')) {
    return hostname;
  }

  const parts = hostname.split('.');
  // for stuff like co.uk / com.au the name is one part further left
  const last = parts[parts.length - 1]!;
  const secondLast = parts[parts.length - 2]!;
  const twoPartEnding = last.length === 2 && ['co', 'com', 'org', 'net', 'ac', 'gov', 'edu'].includes(secondLast);
  const name = parts[parts.length - (twoPartEnding ? 3 : 2)] ?? secondLast;

  return name.charAt(0).toUpperCase() + name.slice(1);
}

// sites serve their favicon at /favicon.ico
function faviconUrl(siteUrl: string): string {
  return new URL('/favicon.ico', siteUrl).href;
}

// list logo = favicon of the site, cow if theres no site / favicon
function entryLogo(entry: Entry): HTMLImageElement {
  const logo = document.createElement('img');
  logo.className = 'entry-logo';
  logo.alt = '';

  if (entry.url) {
    logo.classList.add('favicon');
    logo.src = faviconUrl(entry.url);
    logo.addEventListener('error', () => {
      logo.classList.remove('favicon');
      logo.src = cowurl;
    }, { once: true });
  } else {
    logo.src = cowurl;
  }

  return logo;
}

// same form for adding and editing, with `existing` it edits that entry
function showEntryForm(existing?: Entry) {
  if (!existing) {
    list.querySelector('.entry-item.selected')?.classList.remove('selected');
  }

  // cancel goes back to the entry when editing, otherwise to the cow
  const cancel = () => (existing ? showEntry(existing) : showCow());

  // header
  const heading = document.createElement('h2');
  heading.className = 'detail-title';
  heading.textContent = existing ? 'Edit entry' : 'New password';

  const close = document.createElement('button');
  close.type = 'button';
  close.className = 'detail-close';
  close.textContent = '✕';
  close.title = 'Close';
  close.setAttribute('aria-label', 'Close');
  close.addEventListener('click', cancel);

  const header = document.createElement('div');
  header.className = 'detail-header';
  header.append(heading, close);

  // inputs
  const nameInput = textInput('e.g. GitHub', 'Name');
  const websiteInput = textInput('e.g. github.com', 'Website');
  const usernameInput = textInput('Username or email', 'Username');

  // typed a website but no name yet -> name from the domain
  websiteInput.addEventListener('change', () => {
    const url = normalizeUrl(websiteInput.value);
    if (url && nameInput.value.trim() === '') {
      nameInput.value = siteName(new URL(url).hostname);
      nameInput.classList.remove('invalid');
    }
  });

  const passwordIn = textInput('Password', 'Password');
  passwordIn.type = 'password';
  passwordIn.classList.add('field-input-password');

  // editing -> start with the current values
  if (existing) {
    nameInput.value = existing.title;
    websiteInput.value = existing.url ?? '';
    usernameInput.value = existing.username;
    passwordIn.value = existing.password;
  }

  // show / hide the psw
  const showButton = smallButton('Show', 'Show password');
  const setPasswordVisible = (visible: boolean) => {
    passwordIn.type = visible ? 'text' : 'password';
    showButton.textContent = visible ? 'Hide' : 'Show';
    showButton.title = visible ? 'Hide password' : 'Show password';
    showButton.setAttribute('aria-label', showButton.title);
  };
  showButton.addEventListener('click', () => setPasswordVisible(passwordIn.type === 'password'));

  const passwordLine = document.createElement('div');
  passwordLine.className = 'field-line';
  passwordLine.append(passwordIn, showButton);

  // psw label with generate button on the right
  const passwordLabel = document.createElement('div');
  passwordLabel.className = 'field-label';
  passwordLabel.textContent = 'Password';

  const generate = smallButton('Generate', 'Generate a password');

  const passwordHead = document.createElement('div');
  passwordHead.className = 'field-head';
  passwordHead.append(passwordLabel, generate);

  const passwordRow = document.createElement('div');
  passwordRow.className = 'field';
  passwordRow.append(passwordHead, passwordLine);

  // generate a psw from rust and show the gen settings under it
  generate.addEventListener('click', async () => {
    try {
      await wasmReady;
    } catch (error) {
      console.error('Could not load the WASM module:', error);
      showCopyNotify('Generating failed!');
      return;
    }

    const refresh = () => {
      try {
        passwordIn.value = newPassword();
        passwordIn.classList.remove('invalid');
      } catch (error) {
        console.error('Could not generate password:', error);
        showCopyNotify('Generating failed!');
      }
    };

    const again = smallButton('↻', 'Generate another one');
    again.addEventListener('click', refresh);
    passwordLine.append(again);
    passwordRow.append(generatorSettings(refresh));

    generate.hidden = true;
    setPasswordVisible(true);
    refresh();
    passwordRow.scrollIntoView({ block: 'nearest' });
  });

  const card = document.createElement('div');
  card.className = 'field-card field-form';
  card.append(
    formField('Name', nameInput),
    formField('Website', websiteInput),
    formField('Username', usernameInput),
    passwordRow,
  );

  // cancel / save
  const cancelButton = smallButton('Cancel', 'Cancel');
  cancelButton.addEventListener('click', cancel);

  const saveButton = smallButton('Save', existing ? 'Save changes' : 'Save new password');
  saveButton.type = 'submit'; // so enter in any input saves too
  saveButton.classList.add('primary');

  const buttons = document.createElement('div');
  buttons.className = 'gen-buttons';
  buttons.append(cancelButton, saveButton);

  // real form, so enter submits it
  const form = document.createElement('form');
  form.className = 'detail-content';
  form.noValidate = true; // we check the inputs ourselves
  form.append(header, card, buttons);

  form.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      e.preventDefault(); // keep Esc from closing the popup
      cancel();
    }
  });

  form.addEventListener('submit', (e) => {
    e.preventDefault(); // don't reload the page

    const name = nameInput.value.trim();
    const password = passwordIn.value;
    const websiteText = websiteInput.value.trim();
    const url = websiteText === '' ? undefined : normalizeUrl(websiteText);

    // name + psw are required, website is optional but has to be a real address
    const problems: HTMLInputElement[] = [];
    if (name === '') problems.push(nameInput);
    if (url === null) problems.push(websiteInput);
    if (password === '') problems.push(passwordIn);

    if (problems.length > 0) {
      for (const input of problems) input.classList.add('invalid');
      problems[0]?.focus();
      return;
    }

    const values: Entry = {
      title: name,
      username: usernameInput.value.trim(),
      password,
      url: url ?? undefined,
    };

    // only in memory for now like the mock entries, gone when the popup closes
    let entry: Entry;
    if (existing) {
      Object.assign(existing, values); // change the entry itself, so it stays at its spot in the list
      entry = existing;
    } else {
      mockEntries.push(values);
      entry = values;
    }

    renderList(mockEntries, entry);
    showEntry(entry);
    showCopyNotify(existing ? 'Entry saved!' : 'Password added!');
  });

  detail.replaceChildren(form);
  nameInput.focus();

  // editing -> dont fill in the current tab
  if (existing) return;

  // fill in the current site, only into empty fields
  currentSite().then((site) => {
    if (!site || !form.isConnected) return;
    if (nameInput.value === '') nameInput.value = site.name;
    if (websiteInput.value === '') websiteInput.value = site.url;
    if (document.activeElement === nameInput) nameInput.select();
  });
}
