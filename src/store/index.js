const { createSlice, configureStore } = require('@reduxjs/toolkit')

const downloadablesSlice = createSlice({
  name: 'downloadables',
  initialState: {
    downloadables: []
  },
  reducers: {
    add: (state, { payload }) => {
      const safeAdd = downloadable => {
        const index = state.downloadables.findIndex(
          d => d.path === downloadable.path
        )
        if (index >= 0) {
          state.downloadables.splice(index, 1, downloadable)
          return
        }

        state.downloadables.push(downloadable)
      }

      if (Array.isArray(payload)) {
        payload.forEach(safeAdd)
      } else {
        safeAdd(payload)
      }
    },
    remove: (state, { payload }) => {
      const downloadable = payload
      const index = state.downloadables.findIndex(
        d => d.path === downloadable.path
      )
      state.downloadables.splice(index, 1)
    }
  }
})

const downloadStats = createSlice({
  name: 'downloadStats',
  initialState: {
    stats: []
  },
  reducers: {
    add: (state, { payload }) => {
      const safeAdd = stat => {
        const index = state.stats.findIndex(s => s.id === stat.id)
        if (index >= 0) {
          state.stats.splice(index, 1, stat)
          return
        }

        state.stats.push(stat)
      }

      if (Array.isArray(payload)) {
        payload.forEach(safeAdd)
      } else {
        safeAdd(payload)
      }
    },
    remove: (state, { payload }) => {
      const id = payload
      const index = state.stats.findIndex(s => s.id === id)
      state.stats.splice(index, 1)
    }
  }
})

module.exports = {
  downloadablesStore: configureStore({ reducer: downloadablesSlice.reducer }),
  downloadablesActions: downloadablesSlice.actions,

  downloadStatsStore: configureStore({ reducer: downloadStats.reducer }),
  downloadStatsActions: downloadStats.actions
}
