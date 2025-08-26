import { createPinia } from 'pinia';
import { createApp } from 'vue';
import { ref, nextTick } from 'vue'
import './index.scss';
import App from './App.vue';
import { AuthClient } from "@dfinity/auth-client";
import { project_chatgpt_backend } from 'declarations/project_chatgpt_backend/index';
import { Principal } from "@dfinity/principal";
import './index.css';

const userStartMessages = [
  "Hi, can you help me with a task?",
  "How does blockchain work?",
  "Write me a thank-you email.",
  "Do you have a gift idea for my mom?",
  "What does a dream about water mean?",
  "What are the latest tech news?",
  "I need a recipe for a quick dinner.",
  "Explain to me what inflation is.",
  "Write me a summary of this book.",
  "How can I improve my CV?",
];

function demoColorGetter(y, x) {
  //if (y === x) return '#FF9900';
  return '#E5E7EB'; // jasna szarość
}

function generateImageDescriptor(name, cols, rows, getColorAt) {
  // helper losujący powtarzalny kolor dla (y,x), gdy nie podano getColorAt
  const fallbackColor = (y, x) => {
    // prosty determinizm: hasz z y,x
    const seed = (y * 73856093) ^ (x * 19349663);
    // trzy kanały z seed
    const r = (seed & 0xff);
    const g = ((seed >> 8) & 0xff);
    const b = ((seed >> 16) & 0xff);
    const toHex = (n) => n.toString(16).padStart(2, '0').toUpperCase();
    return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
  };

  const colorAt = typeof getColorAt === 'function' ? getColorAt : fallbackColor;

  const lines = [];
  lines.push(`Content: ${name}`);
  lines.push(`Image:`);
  for (let y = 1; y <= rows; y++) {
    const rowParts = [];
    for (let x = 1; x <= cols; x++) {
      rowParts.push(`|y:${y},x:${x};${colorAt(y, x)}|`);
    }
    lines.push(rowParts.join(''));
  }
  return lines.join('');
}

export const generate = async (ctxName, cols, rows) => {
  const user_date = Date.now();
  const userMsg = { role: 'user', content: generateImageDescriptor(ctxName, cols, rows, demoColorGetter), etc: [user_date, cols, rows] };
  const temp = userMsg.content;
  await addChatMessage(
    loginStatus.value.principal,
    current.value,
    temp,
    'user', cols, rows, user_date
  );
  messages.value.push(userMsg);
  chats.value.find(c => c.id === current.value).msg_len += 1;
  nextTick(() => { scrollToBottom(); });
}

export const endOfMessages = ref(null);

const scrollToBottom = () => {
  endOfMessages.value?.scrollIntoView({ behavior: 'smooth' });
};

export const messages = ref([]);
export const images = ref([]);
export const chats = ref([]);
export const archives = ref([]);
export const current = ref(null);

export const loginStatus = ref({
  loggedIn: false,
  principal: null,
  username: "user",
  icon: "👤",
})

export const login = async () => {
  //loginStatus.value.loggedIn = true;
  //loginStatus.value.principal = Principal.fromText("aaaaa-aa");
  //loginStatus.value.username = await project_chatgpt_backend.get_user_name(loginStatus.value.principal);
  const authClient = await AuthClient.create();

  await authClient.login({
    identityProvider: "https://identity.ic0.app/#authorize",
    onSuccess: async () => {
      const identity = authClient.getIdentity();
      const principal = identity.getPrincipal();
      loginStatus.value.loggedIn = true;
      loginStatus.value.principal = principal;
      loginStatus.value.username = await getUserName(loginStatus.value.principal);
      alert("Logged in as " + loginStatus.value.username);
    },
    onError: (err) => {
      alert("Login failed: " + err);
    },
  });
  await load();
};

export const logout = async () => {
  loginStatus.value.loggedIn = false;
  loginStatus.value.principal = null;
  loginStatus.value.username = "user";
  const authClient = await AuthClient.create();

  // Perform the logout operation
  await authClient.logout();
  messages.value = [];
};

export const rename = async (new_username) => {
  loginStatus.value.username = new_username;
  await project_chatgpt_backend.set_user_name(loginStatus.value.principal, new_username);
};

export const chat = async (message, tag) => {
  if (current.value === null) {
    await create();
  }
  try {
    const size = (messages.value.length > 7) ? 7 : messages.value.length;
    const history = messages.value.slice(-size).map(msg => [msg.role, msg.content]);

    // Dodaj wiadomość użytkownika
    const user_date = Date.now();
    await project_chatgpt_backend.add_chat_message(
      loginStatus.value.principal,
      current.value,
      message,
      'user',
      0,
      0,
      user_date
    );
    messages.value.push({ role: 'user', content: message, etc: [user_date, 0, 0] });
    nextTick(() => scrollToBottom());

    // Dodaj placeholder AI message z loaderem
    const ai_date = Date.now();
    const placeholderIndex = messages.value.push({
      role: tag,
      content: "⏳ AI is thinking...",
      etc: [ai_date, 0, 0],
      loading: true
    }) - 1; // indeks nowego message
    nextTick(() => scrollToBottom());

    let response;
    try {
      response = await project_chatgpt_backend.chat(message, tag, history);
    } catch (err) {
      console.error("AI response error:", err);
      if (err.message && err.message.toLowerCase().includes("timeout")) {
        response = "Something Wrong";
      } else {
        response = "AI service error. Reloading...";
        setTimeout(() => location.reload(), 1500);
      }
    }

    // Zamień placeholder na właściwą odpowiedź
    messages.value[placeholderIndex] = {
      role: tag,
      content: response,
      etc: [ai_date, 0, 0],
      loading: false
    };
    nextTick(() => scrollToBottom());

    // Zapisz odpowiedź AI do backendu
    await project_chatgpt_backend.add_chat_message(
      loginStatus.value.principal,
      current.value,
      response,
      tag,
      0,
      0,
      ai_date
    );

    // Aktualizuj liczbę wiadomości w chatcie
    const chatItem = chats.value.find(c => c.id === current.value);
    if (chatItem) chatItem.msg_len += 2;

  } catch (outerErr) {
    console.error("Critical chat error:", outerErr);
    setTimeout(() => location.reload(), 1500);
  }
};


export const load = async () => {
  if (!loginStatus.value.loggedIn) return;
  chats.value = await project_chatgpt_backend.list_chats(loginStatus.value.principal, false);
}

export const load_archives = async () => {
  if (!loginStatus.value.loggedIn) return;
  archives.value = await project_chatgpt_backend.list_chats(loginStatus.value.principal, true);
}

export const load_images = async () => {
  if (!loginStatus.value.loggedIn) return;
  const result = await project_chatgpt_backend.get_all_images(loginStatus.value.principal);
  if (!result || !result.messages) {
    images.value = [];
    return;
  }
  images.value = result.messages.flatMap(m => [
    { image: m.content, etc: m.etc },
  ]);
}

export const open = async (id) => {
  messages.value = [];
  current.value = id;
  const len = chats.value.find(c => c.id === current.value).msg_len;
  const result = await project_chatgpt_backend.get_chat_history(loginStatus.value.principal, current.value, len);
  if (!result || !result.messages) {
    messages.value = [];
    return;
  }
  messages.value = result.messages.flatMap(m => [
    { role: m.role, content: m.content, etc: m.etc },
  ]);
  nextTick(() => { scrollToBottom(); });
}

export const create = async () => {
  if (!loginStatus.value.loggedIn) return;
  const uuid = crypto.randomUUID();
  const bytes = Uint8Array.from(
    uuid.replace(/-/g, '').match(/.{2}/g).map(b => parseInt(b, 16))
  );
  const name = `New Chat ${chats.value.length + 1}`;
  await project_chatgpt_backend.create_new_chat(loginStatus.value.principal, bytes, name);
  await load();
  current.value = bytes;
}

export const remove_chat = async (id) => {
  await project_chatgpt_backend.delete_chat(loginStatus.value.principal, id);
  if (current.value === id) {
    current.value = null;
    messages.value = [];
  }
  await load();
}

export const rename_chat = async (id, new_name) => {
  await project_chatgpt_backend.rename_chat(loginStatus.value.principal, id, new_name);
  await load();
}

export const archive_chat = async (id, archive) => {
  await project_chatgpt_backend.archive_chat(loginStatus.value.principal, id, archive);
  if (current.value === id) {
    current.value = null;
    messages.value = [];
  }
  await load();
}


export async function updateImage(msgId, new_content) {
  return await project_chatgpt_backend.update_image(loginStatus.value.principal, current.value, msgId, new_content);
}

export async function askAiDraw(query, tag, msg) {
  return await project_chatgpt_backend.askaidraw(query, tag, msg);
}

export async function archiveChat(principal, chatId, archive) {
  return await project_chatgpt_backend.archive_chat(principal, chatId, archive);
}

export async function chatWithBackend(message, width, height, tag, principal, chatId, msgLen) {
  return await project_chatgpt_backend.chat(message, width, height, tag, principal, chatId, msgLen);
}

export async function createNewChat(principal, chatId, name) {
  return await project_chatgpt_backend.create_new_chat(principal, chatId, name);
}

export async function addChatMessage(principal, chatId, content, role, width, height, date) {
  return await project_chatgpt_backend.add_chat_message(principal, chatId, content, role, width, height, date);
}

export async function getChatHistory(principal, chatId, msgLen) {
  return await project_chatgpt_backend.get_chat_history(principal, chatId, msgLen);
}

export async function deleteChat(principal, chatId) {
  return await project_chatgpt_backend.delete_chat(principal, chatId);
}

export async function renameChat(principal, chatId, newName) {
  return await project_chatgpt_backend.rename_chat(principal, chatId, newName);
}

export async function listChats(principal, arch) {
  return await project_chatgpt_backend.list_chats(principal, arch);
}

export async function setUserName(principal, username) {
  return await project_chatgpt_backend.set_user_name(principal, username);
}

export async function getUserName(principal) {
  return await project_chatgpt_backend.get_user_name(principal);
}

export async function tryPrompt(principal) {
  return await project_chatgpt_backend.try_increment_user_prompt(principal);
}

export function getRandomUserMessages() {
  const shuffled = [...userStartMessages].sort(() => 0.5 - Math.random());
  return shuffled.slice(0, 3);
}

createApp(App).use(createPinia()).mount('#app');