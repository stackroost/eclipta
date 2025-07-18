import { useEffect, useState } from 'react'

function App() {
  const [msg, setMsg] = useState("Loading...")

  useEffect(() => {
    fetch('/api/hello')
      .then(res => res.text())
      .then(setMsg)
  }, [])

  return (
    <div style={{ fontFamily: 'sans-serif', padding: '2rem' }}>
      <h1 style={{ fontSize: '2.5rem', marginBottom: '1rem' }}>Welcome to <span style={{ color: '#5e60ce' }}>Eclipta</span></h1>
      <h2 style={{ fontWeight: 'normal' }}>eBPF Observability Dashboard</h2>
      <hr style={{ margin: '1.5rem 0' }} />

      <p><strong>Backend says:</strong> {msg}</p>

      <section style={{ marginTop: '2rem' }}>
        <h3>Real-time Metrics (Coming soon)</h3>
        <ul>
          <li>Network packet trace</li>
          <li>Process activity</li>
          <li>CPU and memory usage</li>
          <li>Filesystem access logs</li>
        </ul>
      </section>
    </div>
  )
}

export default App
