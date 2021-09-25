import React, { useState, useEffect } from 'react';
import publicIp from 'public-ip';

import styles from './IPToolbar.css'

export default () => {
  // TODO:
  const doUsePublicIpAddress = false

  const [ipAddress, setIpAddress] = useState('')
  const [publicIpAddress, setPublicIpAddress] = useState('')

  useEffect(async () => {
    setIpAddress(window.electron.getLocalIpAddress());
    doUsePublicIpAddress && setPublicIpAddress(await publicIp.v4());
  })
  return (<>
    <input className={styles.toolbar} type="text" readOnly={true} value={ipAddress} />
    {doUsePublicIpAddress &&
      <input className={styles.toolbar} type="text" readOnly={true} value={publicIpAddress} />
    }
  </>);
};
