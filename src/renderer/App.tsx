import React from 'react'
import { MemoryRouter as Router, Switch, Route } from 'react-router-dom'
import './App.global.css'
import IPToolbar from './components/IPToolbar'
import Downloadables from './components/Downloadables'
import DownloadStats from './components/DownloadStats'
import UploadForm from './components/UploadForm'

const Home = () => {
  return (
    <main>
      <IPToolbar />
      <Downloadables />
      <DownloadStats />
      <UploadForm style={{ flexGrow: 1 }} />
    </main>
  )
}

export default function App() {
  return (
    <Router>
      <Switch>
        <Route path="/" component={Home} />
      </Switch>
    </Router>
  )
}
