import React, { useCallback } from 'react'
import { useDropzone } from 'react-dropzone'

import styles from './UploadForm.css'

export default ({ style }) => {
  const onDrop = useCallback(acceptedFiles => {
    const downloadablesFormat = acceptedFiles.map(file => ({
      name: file.name,
      path: file.path,
      size: file.size,
      type: file.type
    }))
    window.stores.downloadables.dispatch(
      window.storeActions.downloadables.add(downloadablesFormat)
    )
  }, [])

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop
  })

  return (
    <form className={styles['upload-form']} style={style} {...getRootProps()}>
      <input {...getInputProps()} />
      <label
        className={`${styles['file-input-label']} ${
          isDragActive ? styles['is-dragover'] : ''
        }`}
      >
        <div className={`${styles.content} ${styles['space-y']}`}>
          <svg
            className={styles['download-icon']}
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 50 43"
          >
            <path d="M48.4 26.5c-.9 0-1.7.7-1.7 1.7v11.6h-43.3v-11.6c0-.9-.7-1.7-1.7-1.7s-1.7.7-1.7 1.7v13.2c0 .9.7 1.7 1.7 1.7h46.7c.9 0 1.7-.7 1.7-1.7v-13.2c0-1-.7-1.7-1.7-1.7zm-24.5 6.1c.3.3.8.5 1.2.5.4 0 .9-.2 1.2-.5l10-11.6c.7-.7.7-1.7 0-2.4s-1.7-.7-2.4 0l-7.1 8.3v-25.3c0-.9-.7-1.7-1.7-1.7s-1.7.7-1.7 1.7v25.3l-7.1-8.3c-.7-.7-1.7-.7-2.4 0s-.7 1.7 0 2.4l10 11.6z" />
          </svg>
          <div>
            <strong>Choose a file</strong> or drag it here.
          </div>
        </div>
      </label>
    </form>
  )
}
