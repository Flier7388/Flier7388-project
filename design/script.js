/* ===== colorMXID — точный алгоритм из cinny/src/util/colorMXID.js ===== */
function hashCode(str) {
  let hash = 0;
  for (let i = 0; i < str.length; i += 1) {
    hash = (hash << 5) - hash + str.charCodeAt(i);
    hash |= 0;
  }
  return Math.abs(hash);
}
function colorMXID(userId) {
  const n = (hashCode(userId) % 8) + 1;
  return `var(--mx-uc-${n})`;
}
function initials(name) {
  return name.trim().charAt(0).toUpperCase();
}

/* ===== Данные ===== */
const ROOMS = {
  general: { name: 'general', topic: 'Добро пожаловать в главный чат Matrix HQ', encrypted: false, kind: 'public' },
  dev: { name: 'dev-chat', topic: 'Обсуждение разработки и кода', encrypted: false, kind: 'public' },
  secure: { name: 'secure-room', topic: 'Приватное обсуждение для доверенных участников', encrypted: true, kind: 'invite' },
  announcements: { name: 'announcements', topic: 'Только важные объявления', encrypted: false, kind: 'public' },
  random: { name: 'random', topic: 'Обо всём и ни о чём', encrypted: false, kind: 'public' },
  'rust-general': { name: 'general', topic: 'Обсуждение языка Rust', encrypted: false, kind: 'public' },
  'rust-help': { name: 'help', topic: 'Помощь новичкам', encrypted: false, kind: 'public' },
  'dm-alice': { name: 'alice', topic: '@alice:matrix.org', encrypted: true, kind: 'dm', mxid: '@alice:matrix.org' },
  'dm-bob': { name: 'bob', topic: '@bob:matrix.org', encrypted: true, kind: 'dm', mxid: '@bob:matrix.org' },
};

const UNREAD = {
  dev: { count: 5, highlight: true },
  secure: { count: 3, highlight: false },
};

const PINNED = {
  general: [{ author: 'danya', mxid: '@danya:matrix.org', text: 'Ссылка на документацию sliding sync: spec.matrix.org/proposals/msc3575' }],
};

const roomMessages = {
  general: [
    msg('danya', '@danya:matrix.org', 'привет всем! кто-нибудь пробовал новый sliding sync?', '14:02'),
    msg('danya', '@danya:matrix.org', 'работает прям быстро, синхронизация почти мгновенная', '14:02', [
      { emoji: '👍', count: 3, mine: false },
      { emoji: '🔥', count: 1, mine: false },
    ]),
    msg('alice', '@alice:matrix.org', 'да, мигрировали прошлую неделю. нагрузка на сервер упала почти в 2 раза', '14:05'),
    msg('bot', '@bot:matrix.org', '📊 Статистика комнаты: 284 участника · 1,204 сообщения за неделю', '14:06', [], true),
  ],
  dev: [msg('danya', '@danya:matrix.org', 'кто смотрел последний PR по рефакторингу роутера?', '13:40')],
  secure: [msg('alice', '@alice:matrix.org', 'напоминаю: не постить сюда токены доступа', '12:10')],
  announcements: [msg('bot', '@bot:matrix.org', '📣 Завтра плановые работы на сервере с 02:00 до 04:00 UTC', '09:00', [], true)],
  random: [],
  'rust-general': [msg('mira', '@mira:matrix.org', 'кто-нибудь уже щупал async closures в стабильном Rust?', '11:20')],
  'rust-help': [],
  'dm-alice': [msg('alice', '@alice:matrix.org', 'привет! как продвигается миграция?', '10:12')],
  'dm-bob': [],
};

function msg(author, mxid, text, time, reactions, bot) {
  return { author, mxid, text, time, reactions: reactions || [], bot: !!bot };
}

const BOT_REPLIES = [
  'интересная мысль 🤔',
  'видел похожее обсуждение в #dev-chat',
  '+1',
  'а вы пробовали перезайти в комнату? иногда помогает пересинхронизировать state',
  'напоминаю, что эта комната зашифрована — только участники видят содержимое',
];

let currentRoomId = 'general';
let currentTarget = 'home';

/* ===== Рельса ===== */
const railItems = document.querySelectorAll('.rail-item[data-target]');
railItems.forEach((item) => {
  item.addEventListener('click', () => setActiveTarget(item.dataset.target));
});

function setActiveTarget(target) {
  currentTarget = target;
  railItems.forEach((i) => i.classList.toggle('is-active', i.dataset.target === target));
  renderLobby(target);
}

/* ===== Лобби ===== */
const lobbyHeader = document.getElementById('lobbyHeader');
const lobbyBody = document.getElementById('lobbyBody');

function renderLobby(target) {
  if (target === 'home') {
    lobbyHeader.innerHTML = '<h1>Home</h1>';
    lobbyBody.innerHTML = '';
    lobbyBody.appendChild(buildCategory('Rooms', ['general', 'dev', 'secure', 'announcements', 'random']));
  } else if (target === 'direct') {
    lobbyHeader.innerHTML = '<h1>Direct Messages</h1>';
    lobbyBody.innerHTML = '';
    lobbyBody.appendChild(buildDmList(['dm-alice', 'dm-bob']));
  } else if (target === 'space:hq') {
    lobbyHeader.innerHTML = `<span class="avatar avatar--user" style="--mx:var(--mx-uc-7)">M</span><h1>Matrix HQ</h1>`;
    lobbyBody.innerHTML = '';
    lobbyBody.appendChild(buildCategory('Rooms', ['general', 'dev', 'secure', 'announcements', 'random']));
  } else if (target === 'space:rust') {
    lobbyHeader.innerHTML = `<span class="avatar avatar--user" style="--mx:var(--mx-uc-4)">R</span><h1>Rust Lang</h1>`;
    lobbyBody.innerHTML = '';
    lobbyBody.appendChild(buildCategory('Rooms', ['rust-general', 'rust-help']));
  } else if (target === 'explore') {
    lobbyHeader.innerHTML = '<h1>Explore</h1>';
    lobbyBody.innerHTML = `
      <div class="lobby-search">
        <svg class="icon"><use href="#i-search"/></svg>
        <input type="text" placeholder="Найти публичную комнату">
      </div>
      <p class="lobby-placeholder">Здесь будут отображаться публичные комнаты вашего домашнего сервера.</p>`;
  } else if (target === 'inbox') {
    lobbyHeader.innerHTML = '<h1>Inbox</h1>';
    lobbyBody.innerHTML = '';
    lobbyBody.appendChild(buildInbox());
  }
}

function buildCategory(label, roomIds) {
  const cat = document.createElement('div');
  cat.className = 'lobby-cat';
  const btn = document.createElement('button');
  btn.className = 'lobby-cat-btn';
  btn.innerHTML = `<svg class="icon"><use href="#i-chevron-down"/></svg><span>${label}</span>`;
  const body = document.createElement('div');
  body.className = 'lobby-cat-body';

  roomIds.forEach((id) => body.appendChild(buildRoomRow(id)));

  btn.addEventListener('click', () => {
    cat.classList.toggle('is-closed');
    const use = btn.querySelector('use');
    use.setAttribute('href', cat.classList.contains('is-closed') ? '#i-chevron-right' : '#i-chevron-down');
  });

  cat.appendChild(btn);
  cat.appendChild(body);
  return cat;
}

function buildRoomRow(id) {
  const room = ROOMS[id];
  const row = document.createElement('div');
  const unread = UNREAD[id];
  row.className = 'room-row' + (id === currentRoomId ? ' is-active' : '') + (unread ? ' has-unread' : '');

  const icon = room.kind === 'invite' ? 'i-lock' : 'i-hash';
  row.innerHTML = `
    <span class="room-avatar"><svg class="icon"><use href="#${icon}"/></svg></span>
    <span class="room-row__name">${room.name}</span>
    ${unread ? `<span class="unread-pill${unread.highlight ? ' is-highlight' : ''}">${unread.count}</span>` : ''}
  `;
  row.addEventListener('click', () => selectRoom(id));
  return row;
}

function buildDmList(ids) {
  const wrap = document.createElement('div');
  wrap.className = 'lobby-cat-body';
  ids.forEach((id) => {
    const room = ROOMS[id];
    const row = document.createElement('div');
    row.className = 'room-row dm-row' + (id === currentRoomId ? ' is-active' : '');
    row.innerHTML = `
      <span class="avatar avatar--user" style="--mx:${colorMXID(room.mxid)}">${initials(room.name)}</span>
      <span class="room-row__name">${room.name}</span>
    `;
    row.addEventListener('click', () => selectRoom(id));
    wrap.appendChild(row);
  });
  return wrap;
}

function buildInbox() {
  const wrap = document.createElement('div');
  const invites = [
    { room: 'design-talk', from: '@kate:matrix.org' },
    { room: 'opensource-kz', from: '@ivan:matrix.org' },
  ];
  invites.forEach((inv, idx) => {
    const card = document.createElement('div');
    card.className = 'invite-card';
    card.innerHTML = `
      <span class="room-avatar"><svg class="icon"><use href="#i-hash"/></svg></span>
      <span class="invite-card__meta">
        <span class="invite-card__name">${inv.room}</span>
        <span class="invite-card__sub">Приглашение от ${inv.from}</span>
      </span>
      <button class="chip chip--primary" data-accept>Accept</button>
      <button class="chip chip--ghost" data-decline>Decline</button>
    `;
    card.querySelectorAll('[data-accept],[data-decline]').forEach((btn) => {
      btn.addEventListener('click', () => card.remove());
    });
    wrap.appendChild(card);
  });
  return wrap;
}

renderLobby('home');

/* ===== Выбор комнаты ===== */
const roomAvatarEl = document.getElementById('roomAvatar');
const roomNameEl = document.getElementById('roomName');
const roomTopicEl = document.getElementById('roomTopic');
const headerSearchBtn = document.getElementById('headerSearchBtn');
const pinBadge = document.getElementById('pinBadge');
const composerBox = document.getElementById('composerBox');

function selectRoom(id) {
  currentRoomId = id;
  const room = ROOMS[id];

  roomNameEl.textContent = room.name;
  roomTopicEl.textContent = room.topic;
  composerBox.dataset.placeholder = `Message ${room.kind === 'dm' ? room.name : '#' + room.name}`;

  if (room.kind === 'dm') {
    roomAvatarEl.innerHTML = '';
    roomAvatarEl.style.background = colorMXID(room.mxid);
    roomAvatarEl.style.color = 'var(--surface-container)';
    roomAvatarEl.textContent = initials(room.name);
  } else {
    roomAvatarEl.style.background = '';
    roomAvatarEl.style.color = '';
    const icon = room.kind === 'invite' ? 'i-lock' : 'i-hash';
    roomAvatarEl.innerHTML = `<svg class="icon"><use href="#${icon}"/></svg>`;
  }

  headerSearchBtn.style.display = room.encrypted ? 'none' : '';

  const pinned = PINNED[id] || [];
  pinBadge.hidden = pinned.length === 0;
  pinBadge.textContent = pinned.length;

  renderLobby(currentTarget);
  renderTimeline(id);
  hideTyping();
}

/* ===== Таймлайн ===== */
const timelineEl = document.getElementById('timeline');

function renderTimeline(id) {
  timelineEl.innerHTML = '';
  const messages = roomMessages[id] || [];
  messages.forEach((m, idx) => {
    const collapsed = idx > 0 && messages[idx - 1].author === m.author;
    timelineEl.appendChild(buildMessageNode(m, collapsed));
  });
  scrollToBottom();
}

function buildMessageNode(m, collapsed, isNew) {
  const el = document.createElement('div');
  el.className = 'msg' + (collapsed ? ' is-collapsed' : '') + (isNew ? ' is-new' : '');
  const color = colorMXID(m.mxid);

  const reactionsHtml = m.reactions.length
    ? `<div class="reactions">${m.reactions
        .map((r) => `<span class="reaction${r.mine ? ' is-mine' : ''}" data-emoji="${r.emoji}">${r.emoji} ${r.count}</span>`)
        .join('')}</div>`
    : '';

  el.innerHTML = `
    <div class="msg__gutter">
      ${collapsed
        ? `<span class="msg__gutter-time">${m.time}</span>`
        : `<span class="avatar avatar--user" style="--mx:${color}">${initials(m.author)}</span>`}
    </div>
    <div class="msg__content">
      ${
        collapsed
          ? ''
          : `<div class="msg__head">
              <span class="msg__author" style="color:${color}">${m.author}</span>
              <span class="msg__mxid">${m.mxid}</span>
              <span class="msg__sep">|</span>
              <span class="msg__time">${m.time}</span>
            </div>`
      }
      <div class="msg__text">
        ${m.text}
        <div class="msg__toolbar">
          <button data-react title="React"><svg class="icon"><use href="#i-smile"/></svg></button>
          <button title="Reply"><svg class="icon"><use href="#i-link"/></svg></button>
          <button title="More"><svg class="icon"><use href="#i-dots"/></svg></button>
        </div>
      </div>
      ${reactionsHtml}
    </div>
  `;

  el.querySelectorAll('.reaction').forEach((reactionEl) => {
    const emoji = reactionEl.dataset.emoji;
    reactionEl.addEventListener('click', () => toggleReaction(m, emoji));
  });

  el.querySelector('[data-react]').addEventListener('click', () => toggleReaction(m, '👍'));

  return el;
}

function toggleReaction(m, emoji) {
  let r = m.reactions.find((x) => x.emoji === emoji);
  if (!r) {
    r = { emoji, count: 0, mine: false };
    m.reactions.push(r);
  }
  r.mine = !r.mine;
  r.count += r.mine ? 1 : -1;
  if (r.count <= 0) m.reactions = m.reactions.filter((x) => x !== r);
  renderTimeline(currentRoomId);
}

function scrollToBottom() {
  timelineEl.scrollTop = timelineEl.scrollHeight;
}

/* ===== Индикатор печати ===== */
const typingRow = document.getElementById('typingRow');
const typingText = document.getElementById('typingText');
document.getElementById('typingDropBtn').addEventListener('click', hideTyping);

function showTyping(name) {
  typingText.innerHTML = `<b>${name}</b> is typing...`;
  typingRow.hidden = false;
}
function hideTyping() {
  typingRow.hidden = true;
}

/* ===== Отправка сообщений ===== */
const sendBtn = document.getElementById('sendBtn');
const emojiBtn = document.getElementById('emojiBtn');

emojiBtn.addEventListener('click', () => {
  composerBox.focus();
  document.execCommand('insertText', false, '🙂');
});

function sendCurrentMessage() {
  const text = composerBox.innerText.trim();
  if (!text) return;

  const list = roomMessages[currentRoomId];
  const last = list[list.length - 1];
  const collapsed = !!last && last.author === 'yulia';
  const time = new Date().toLocaleTimeString('ru-RU', { hour: '2-digit', minute: '2-digit' });

  const m = msg('yulia', '@yulia:matrix.org', escapeHtml(text), time);
  list.push(m);
  timelineEl.appendChild(buildMessageNode(m, collapsed, true));
  scrollToBottom();
  composerBox.textContent = '';

  triggerBotReply(text);
}

function triggerBotReply(userText) {
  const lower = userText.toLowerCase();
  let reply = BOT_REPLIES[Math.floor(Math.random() * BOT_REPLIES.length)];

  if (lower.includes('wake up') || lower.includes('neo') || lower.includes('нео')) {
    reply = 'Wake up, Neo… 🐇 Follow the white rabbit.';
  } else if (lower.includes('матрица')) {
    reply = 'Открой свой разум. Matrix — протокол, а не фильм... или всё-таки нет? 🕶';
  }

  showTyping('bot');
  setTimeout(() => {
    hideTyping();
    const list = roomMessages[currentRoomId];
    const last = list[list.length - 1];
    const collapsed = !!last && last.author === 'bot';
    const time = new Date().toLocaleTimeString('ru-RU', { hour: '2-digit', minute: '2-digit' });
    const m = msg('bot', '@bot:matrix.org', reply, time, [], true);
    list.push(m);
    timelineEl.appendChild(buildMessageNode(m, collapsed, true));
    scrollToBottom();
  }, 900 + Math.random() * 700);
}

function escapeHtml(str) {
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}

sendBtn.addEventListener('click', sendCurrentMessage);
composerBox.addEventListener('keydown', (e) => {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    sendCurrentMessage();
  }
});

/* ===== Members drawer ===== */
const membersDrawer = document.getElementById('membersDrawer');
document.getElementById('membersBtn').addEventListener('click', () => {
  membersDrawer.hidden = !membersDrawer.hidden;
});

/* ===== Topic overlay ===== */
const topicOverlay = document.getElementById('topicOverlay');
roomTopicEl.addEventListener('click', () => {
  document.getElementById('topicDialogName').textContent = ROOMS[currentRoomId].name;
  document.getElementById('topicDialogText').textContent = ROOMS[currentRoomId].topic;
  openOverlay(topicOverlay);
});
document.getElementById('closeTopicBtn').addEventListener('click', () => closeOverlay(topicOverlay));

/* ===== Settings overlay ===== */
const settingsOverlay = document.getElementById('settingsOverlay');
const settingsSectionContent = document.getElementById('settingsSectionContent');
const SETTINGS_CONTENT = {
  account: { title: 'Account', body: 'Управление профилем: отображаемое имя, аватар, email и привязанные номера телефона.' },
  appearance: { title: 'Appearance', body: 'Тема оформления, размер шрифта, плотность сообщений и язык интерфейса.' },
  notifications: { title: 'Notifications', body: 'Глобальные правила уведомлений, звуки и исключения по комнатам.' },
  voice: { title: 'Voice & Video', body: 'Выбор микрофона, камеры и настройки качества звонков.' },
  logout: { title: 'Logout', body: 'Выйти из этой сессии на этом устройстве.' },
};

const settingsTabBtn = document.getElementById('settingsTab');
settingsTabBtn.addEventListener('click', () => {
  setSettingsSection('account');
  openOverlay(settingsOverlay);
  settingsTabBtn.classList.add('is-active');
});
document.getElementById('closeSettingsBtn').addEventListener('click', () => {
  closeOverlay(settingsOverlay);
  settingsTabBtn.classList.remove('is-active');
});

document.querySelectorAll('.dialog__nav-item').forEach((item) => {
  item.addEventListener('click', () => setSettingsSection(item.dataset.section));
});

function setSettingsSection(section) {
  document.querySelectorAll('.dialog__nav-item').forEach((i) => i.classList.toggle('is-active', i.dataset.section === section));
  const c = SETTINGS_CONTENT[section];
  settingsSectionContent.innerHTML = `<h2>${c.title}</h2><p>${c.body}</p>`;
}

/* ===== Search overlay ===== */
const searchOverlay = document.getElementById('searchOverlay');
const searchTabBtn = document.getElementById('searchTab');
searchTabBtn.addEventListener('click', () => {
  openOverlay(searchOverlay);
  searchTabBtn.classList.add('is-active');
});

/* ===== Feedback overlay ===== */
const feedbackOverlay = document.getElementById('feedbackOverlay');
const feedbackTabBtn = document.getElementById('feedbackTab');
const starRating = document.getElementById('starRating');
const feedbackText = document.getElementById('feedbackText');
const feedbackStatus = document.getElementById('feedbackStatus');
const submitFeedbackBtn = document.getElementById('submitFeedbackBtn');
let feedbackRating = 0;

function renderStars(value) {
  starRating.querySelectorAll('.star-btn').forEach((btn) => {
    btn.classList.toggle('is-filled', Number(btn.dataset.value) <= value);
  });
}

starRating.querySelectorAll('.star-btn').forEach((btn) => {
  btn.addEventListener('click', () => {
    feedbackRating = Number(btn.dataset.value);
    renderStars(feedbackRating);
  });
  btn.addEventListener('mouseenter', () => renderStars(Number(btn.dataset.value)));
});
starRating.addEventListener('mouseleave', () => renderStars(feedbackRating));

function resetFeedbackForm() {
  feedbackRating = 0;
  feedbackText.value = '';
  feedbackStatus.textContent = '';
  submitFeedbackBtn.disabled = false;
  renderStars(0);
}

feedbackTabBtn.addEventListener('click', () => {
  resetFeedbackForm();
  openOverlay(feedbackOverlay);
  feedbackTabBtn.classList.add('is-active');
});

document.getElementById('closeFeedbackBtn').addEventListener('click', () => closeOverlay(feedbackOverlay));

submitFeedbackBtn.addEventListener('click', () => {
  if (feedbackRating === 0) {
    feedbackStatus.style.color = 'var(--critical-main)';
    feedbackStatus.textContent = 'Поставьте оценку';
    return;
  }
  feedbackStatus.style.color = 'var(--success-main)';
  feedbackStatus.textContent = 'Спасибо за отзыв!';
  submitFeedbackBtn.disabled = true;
  setTimeout(() => closeOverlay(feedbackOverlay), 1100);
});

/* ===== Overlay helpers ===== */
function openOverlay(el) {
  el.classList.add('is-open');
}
function closeOverlay(el) {
  el.classList.remove('is-open');
  settingsTabBtn.classList.remove('is-active');
  searchTabBtn.classList.remove('is-active');
  feedbackTabBtn.classList.remove('is-active');
}
document.querySelectorAll('.overlay').forEach((overlay) => {
  overlay.addEventListener('click', (e) => {
    if (e.target === overlay) closeOverlay(overlay);
  });
});
document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') document.querySelectorAll('.overlay.is-open').forEach(closeOverlay);
});

/* ===== Попауты (Create / Room menu / Pin) ===== */
function openPopoutAt(popout, trigger, align) {
  const wasOpen = popout.classList.contains('is-open');
  document.querySelectorAll('.popout.is-open').forEach((p) => p.classList.remove('is-open'));
  if (wasOpen) return;

  popout.classList.add('is-open');
  const rect = trigger.getBoundingClientRect();
  const popRect = popout.getBoundingClientRect();

  if (align === 'right-of') {
    popout.style.left = `${rect.right + 8}px`;
    popout.style.top = `${rect.top}px`;
  } else {
    let left = rect.right - popRect.width;
    if (left < 8) left = 8;
    popout.style.left = `${left}px`;
    popout.style.top = `${rect.bottom + 6}px`;
  }
}

document.addEventListener('click', (e) => {
  document.querySelectorAll('.popout.is-open').forEach((p) => {
    if (!p.contains(e.target) && !e.target.closest('#createTab, #roomMenuBtn, #pinBtn')) {
      p.classList.remove('is-open');
    }
  });
});

const createPopout = document.getElementById('createPopout');
document.getElementById('createTab').addEventListener('click', (e) => {
  e.stopPropagation();
  openPopoutAt(createPopout, e.currentTarget, 'right-of');
});

const roomMenuPopout = document.getElementById('roomMenuPopout');
document.getElementById('roomMenuBtn').addEventListener('click', (e) => {
  e.stopPropagation();
  openPopoutAt(roomMenuPopout, e.currentTarget);
});
document.getElementById('markReadItem').addEventListener('click', () => {
  roomMenuPopout.classList.remove('is-open');
});

const pinPopout = document.getElementById('pinPopout');
const pinPopoutContent = document.getElementById('pinPopoutContent');
document.getElementById('pinBtn').addEventListener('click', (e) => {
  e.stopPropagation();
  const pinned = PINNED[currentRoomId] || [];
  pinPopoutContent.innerHTML = pinned.length
    ? pinned.map((p) => `<div style="margin-bottom:8px"><b style="color:${colorMXID(p.mxid)}">${p.author}</b><br>${p.text}</div>`).join('')
    : 'No pinned messages';
  openPopoutAt(pinPopout, e.currentTarget);
});

/* ===== Инициализация ===== */
selectRoom('general');
