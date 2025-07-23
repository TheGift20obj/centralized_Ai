<script setup>
import { ref } from 'vue';
import { project_chatgpt_backend } from 'declarations/project_chatgpt_backend/index';
import { login, chatWithBackend } from './main.js';

const messages = ref([]);
const userInput = ref('');

async function sendMessage() {
  messages.value.push({ role: 'user', content: userInput.value });

  try {
    const reply = await chatWithBackend(userInput.value);
    messages.value.push({ role: 'ai', content: reply });
  } catch (error) {
    messages.value.push({ role: 'ai', content: 'Błąd połączenia! ' + error.message });
    console.error('Chat error:', error);
  }

  userInput.value = '';
}
</script>

<template>
  <main>
    <div v-for="msg in messages" :key="msg.content" :class="msg.role">
      <strong>{{ msg.role }}:</strong> {{ msg.content }}
    </div>
    <input v-model="userInput" @keyup.enter="sendMessage" placeholder="Napisz wiadomość..." />
    <button @click="sendMessage">Wyślij</button>
    <button @click="login">Zaloguj się przez Internet Identity</button>
  </main>
</template>