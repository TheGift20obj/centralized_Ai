<template>
  <!-- Sidebar -->
  <div
    :class="[
      'bg-gray-900 text-white transition-all duration-300 flex flex-col h-screen justify-between',
      isOpen ? 'w-64' : 'w-16'
    ]"
  >
    <!-- Top actions and chat list (only visible if logged in) -->
    <div class="flex-1 min-h-0 flex flex-col">
      <!-- Collapse button -->
      <div class="p-2 border-b border-gray-800">
        <button
          class="flex items-center w-full text-left hover:bg-gray-800 p-2 rounded"
          @click="toggleSidebar"
        >
          <span class="w-5 text-center">{{ isOpen ? '◀' : '▶' }}</span>
          <span v-if="isOpen" class="ml-3">Collapse</span>
        </button>
      </div>

      <!-- Only show actions and chat list if logged in -->
      <div v-if="loginStatus.loggedIn" class="flex-1 min-h-0 flex flex-col">
        <!-- Top actions -->
        <div class="p-2 space-y-1 border-b border-gray-800">
          <button
            class="flex items-center w-full text-left hover:bg-gray-800 p-2 rounded"
            @click="newChat"
          >
            <span class="w-5 text-center">＋</span>
            <span v-if="isOpen" class="ml-3">New Chat</span>
          </button>

          <button
            class="flex items-center w-full text-left hover:bg-gray-800 p-2 rounded"
            @click="$emit('search')"
          >
            <span class="w-5 text-center">🔍</span>
            <span v-if="isOpen" class="ml-3">Search</span>
          </button>

          <button
            class="flex items-center w-full text-left hover:bg-gray-800 p-2 rounded"
            @click="$emit('open-library')"
          >
            <span class="w-5 text-center">📚</span>
            <span v-if="isOpen" class="ml-3">Library</span>
          </button>
          <button
            class="flex items-center w-full text-left hover:bg-gray-800 p-2 rounded"
            @click="$emit('open-archives')"
          >
            <span class="w-5 text-center">📦</span>
            <span v-if="isOpen" class="ml-3">Archives</span>
          </button>
        </div>

        <!-- Scrollable chat list -->
        <div v-if="isOpen" class="flex-1 min-h-0 flex flex-col p-2 overflow-hidden">
            <h2 class="text-xs uppercase tracking-wide text-gray-400 px-2 mb-2">
                Chats
            </h2>

            <section class="flex-1 min-h-0 overflow-y-auto pr-1">
                <ul class="space-y-1">
                    <li
                        v-for="chat in chats"
                        :key="chat.id"
                        :class="[
                        'p-2 rounded cursor-pointer hover:bg-gray-800 flex justify-between items-center',
                        chat.id === current ? 'bg-gray-700 text-white' : 'text-gray-200'
                        ]"
                    >
                        <span 
                        class="truncate flex-1" 
                        @click="openChat(chat.id)"
                        >
                        {{ chat.name }}
                        </span>

                        <!-- Actions -->
                        <div class="relative">
                            <button 
                                class="px-2 py-1 rounded hover:bg-gray-700" 
                                @click="showMenu = showMenu === chat.id ? null : chat.id"
                            >
                                ⋮
                            </button>

                            <!-- Dropdown -->
                            <div 
                                v-if="showMenu === chat.id" 
                                class="absolute right-0 mt-1 w-40 bg-gray-800 rounded shadow-lg z-10"
                            >
                                <button 
                                @click="renameChatPrompt(chat.id, chat.name)" 
                                class="block w-full text-left px-3 py-2 hover:bg-gray-700"
                                >
                                ✏️&nbsp;Rename
                                </button>
                                <button 
                                @click="archiveChatAction(chat.id, true)" 
                                class="block w-full text-left px-3 py-2 hover:bg-gray-700"
                                >
                                📦&nbsp;Archive
                                </button>
                                <button 
                                @click="removeChat(chat.id)" 
                                class="block w-full text-left px-3 py-2 text-red-400 hover:bg-gray-700"
                                >
                                🗑️&nbsp;Delete
                                </button>
                            </div>
                        </div>
                    </li>
                </ul>
            </section>
        </div>
      </div>
    </div>

    <!-- User section at bottom -->
    <div class="p-2 border-t border-gray-800 relative">
      <button
        class="flex items-center w-full hover:bg-gray-800 p-2 rounded"
        @click="toggleUserMenu"
      >
        <span class="w-8 h-8 bg-gray-700 rounded-full flex items-center justify-center">
          {{ loginStatus.icon || '👤' }}
        </span>
        <span v-if="isOpen" class="ml-3 truncate">
          {{ loginStatus.loggedIn ? loginStatus.username : 'Guest' }}
        </span>
      </button>

      <!-- Dropdown menu -->
      <div
        v-if="showUserMenu"
        class="absolute bottom-12 left-0 w-full bg-gray-800 rounded shadow-md z-10"
      >
        <div v-if="!loginStatus.loggedIn">
          <button
            class="w-full text-left px-4 py-2 hover:bg-gray-700 rounded"
            @click="loginUser"
          >
            Login
          </button>
        </div>
        <div v-else>
          <button
            class="w-full text-left px-4 py-2 hover:bg-gray-700 rounded"
            @click="changeName"
          >
            Change Name
          </button>
          <button
            class="w-full text-left px-4 py-2 hover:bg-gray-700 rounded"
            @click="changeIcon"
          >
            Change Icon
          </button>
          <button
            class="w-full text-left px-4 py-2 hover:bg-gray-700 rounded"
            @click="logoutUser"
          >
            Logout
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { loginStatus, login, logout, rename, chats, create, open, current, rename_chat, remove_chat, archive_chat } from './main.js'

// Sidebar state
const isOpen = ref(false)
const showUserMenu = ref(false)

const showMenu = ref(null);

// Methods
function toggleSidebar() {
  isOpen.value = !isOpen.value
  if (!isOpen.value) {
    showUserMenu.value = false
    showMenu.value = null
  }
}

function toggleUserMenu() {
  isOpen.value = true
  showUserMenu.value = !showUserMenu.value
}

async function newChat() {
  await create()
}

function search() {
  alert("Search clicked!")
}

function openLibrary() {
  alert("Library opened!")
}

function openArchives() {
  alert("Archives opened!")
}

async function renameChatPrompt(id, oldName) {
  const newName = prompt("Enter new chat name:", oldName)
  if (newName && newName !== oldName) {
    await rename_chat(id, newName)
  }
  showMenu.value = null
}

async function archiveChatAction(id, archive) {
  await archive_chat(id, archive)
  showMenu.value = null
}

async function removeChat(id) {
  if (confirm("Are you sure you want to delete this chat? This action cannot be undone.")) {
    await remove_chat(id)
  }
  showMenu.value = null
}

async function openChat(id) {
  await open(id)
}

async function loginUser() {
  await login()
  showUserMenu.value = false
}

async function logoutUser() {
  await logout()
  showUserMenu.value = false
}

async function changeName() {
  const newName = prompt("Enter new name:", loginStatus.username)
  if (newName) await rename(newName)
  showUserMenu.value = false
}

function changeIcon() {
  const newIcon = prompt("Enter new emoji for icon:", loginStatus.icon || '')
  if (newIcon) loginStatus.icon = newIcon
  showUserMenu.value = false
}
</script>

<style scoped>
/* Scrollbar styling */
::-webkit-scrollbar {
  width: 6px;
}
::-webkit-scrollbar-thumb {
  background: #555;
  border-radius: 4px;
}
::-webkit-scrollbar-thumb:hover {
  background: #777;
}

.chat-list-container {
  display: flex;
  max-height: 100%; /* Cały sidebar maksymalnie 80% wysokości ekranu */
  flex-grow: 1;
  overflow-y: auto;
  flex-direction: column;
  scrollbar-width: thin;
}
</style>
