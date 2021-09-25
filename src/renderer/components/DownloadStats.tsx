import React, { useState, useEffect } from 'react'

import styles from './DownloadStats.module.scss'

const DownloadStats = () => {
  const [downloadStats, setDownloadStats] = useState({})

  useEffect(() =>
    window.stores.downloadStats.subscribe(() =>
      setDownloadStats(() =>
        window.lodash.groupBy(
          window.stores.downloadStats.getState().stats,
          'downloadable.path'
        )
      )
    )
  )

  return (
    Object.keys(downloadStats).length > 0 && (
      <div className={styles.list}>
        {Object.values(downloadStats).map((stats, index) => (
          <div key={index} className={styles.stat}>
            <label>{stats[0].downloadable.name}</label>
            {stats.map(stat => (
              <div
                key={stat.id}
                className={styles.progress}
                title={`${stat.ip} [${(stat.uploadedSize / 1000000).toFixed(
                  2
                )}MB/${(stat.downloadable.size / 1000000).toFixed(2)}MB]`}
                style={{
                  '--width': `${
                    (stat.uploadedSize / stat.downloadable.size) * 100
                  }%`
                }}
              />
            ))}
          </div>
        ))}
      </div>
    )
  )
}

export default DownloadStats
