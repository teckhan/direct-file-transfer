const { contextBridge, ipcRenderer } = require('electron')
const os = require('os')
const { groupBy } = require('lodash')
const {
  downloadablesStore,
  downloadablesActions,
  downloadStatsStore,
  downloadStatsActions
} = require('../store')
const start = require('../server')

start()

// inject lodash
contextBridge.exposeInMainWorld('lodash', {
  groupBy
})

// inject store
contextBridge.exposeInMainWorld('stores', {
  downloadables: downloadablesStore,
  downloadStats: downloadStatsStore
})
contextBridge.exposeInMainWorld('storeActions', {
  downloadables: downloadablesActions,
  downloadStats: downloadStatsActions
})

contextBridge.exposeInMainWorld('electron', {
  ipcRenderer: {
    myPing() {
      ipcRenderer.send('ipc-example', 'ping')
    },
    on(channel, func) {
      const validChannels = ['ipc-example']
      if (validChannels.includes(channel)) {
        // Deliberately strip event as it includes `sender`
        ipcRenderer.on(channel, (event, ...args) => func(...args))
      }
    },
    once(channel, func) {
      const validChannels = ['ipc-example']
      if (validChannels.includes(channel)) {
        // Deliberately strip event as it includes `sender`
        ipcRenderer.once(channel, (event, ...args) => func(...args))
      }
    }
  },
  getLocalIpAddress() {
    return os.networkInterfaces().en0.find(e => e.family === 'IPv4').address
  }
})
