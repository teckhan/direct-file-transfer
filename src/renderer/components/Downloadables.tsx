import React, { useState, useEffect } from 'react'

import styles from './Downloadables.module.scss'

const Downloadables = () => {
  const [downloadables, setDownloadables] = useState([])

  useEffect(() => {
    return window.stores.downloadables.subscribe(() =>
      setDownloadables(
        () => window.stores.downloadables.getState().downloadables
      )
    )
  })

  const remove = downloadable => {
    window.stores.downloadables.dispatch(
      window.storeActions.downloadables.remove(downloadable)
    )
  }

  const removeAll = () => {
    downloadables.forEach(downloadable => {
      window.stores.downloadables.dispatch(
        window.storeActions.downloadables.remove(downloadable)
      )
    })
  }

  return (
    downloadables.length > 0 && (
      <div className={styles.list}>
        {downloadables.map((downloadable, index) => (
          <button
            key={index}
            className={styles.button}
            onClick={() =>
              confirm(`Delete file "${downloadable.name}"?`) &&
              remove(downloadable)
            }
            type="button"
            title={downloadable.path}
          >
            <div className={styles.downloadable}>
              <div>{downloadable.name}</div>
              <span className="material-icons">clear</span>
            </div>
          </button>
        ))}
        {downloadables.length > 1 && (
          <button
            className={styles.removeAll}
            onClick={() => confirm(`Delete'emm all?`) && removeAll()}
            type="button"
          >
            <div className={styles.downloadable}>
              <div>Clear All</div>
            </div>
          </button>
        )}
      </div>
    )
  )
}

export default Downloadables
