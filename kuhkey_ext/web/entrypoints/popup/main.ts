import './style.css';
import cowurl from '../../assets/cow.jpeg';
import { mockEntries, type Entry } from '../../components/test';

const MASK = '🐄🐮🐄🐮🐄🐮🐄🐮';

const list = document.querySelector<HTMLElement>('#entries')!;
const detail = document.querySelector<HTMLElement>('#detail')!;

renderList(mockEntries);
showCow();

function renderList(entries: Entry[]) {
  list.replaceChildren();

  for (const entry of entries) {
    const item = document.createElement('button');
    item.className = 'entry-item';

    const logo = document.createElement('img');
    logo.className = 'entry-logo';
    logo.src = cowurl;
    logo.alt = '';

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

    list.append(item);
  }
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
  close.title = 'Schließen';
  close.setAttribute('aria-label', 'Schließen');
  close.addEventListener('click', () => {
    list.querySelector('.entry-item.selected')?.classList.remove('selected');
    showCow();
  });

  const header = document.createElement('div');
  header.className = 'detail-header';
  header.append(heading, close);

  const card = document.createElement('div');
  card.className = 'field-card';
  card.append(
    field('Benutzername', entry.username),
    passwordField(entry.password),
  );

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

function passwordField(password: string): HTMLElement {
  const row = field('Passwort', MASK);
  const valueEl = row.querySelector<HTMLElement>('.field-value')!;
  valueEl.classList.add('field-password');
  valueEl.tabIndex = 0; // so it can be reached with tab

  // so it only reveals the password when a user intends to do it
  const reveal = () => (valueEl.textContent = password);
  const hide = () => (valueEl.textContent = MASK);
  valueEl.addEventListener('mouseenter', reveal);
  valueEl.addEventListener('mouseleave', hide);
  valueEl.addEventListener('focus', reveal);
  valueEl.addEventListener('blur', hide);

  return row;
}
