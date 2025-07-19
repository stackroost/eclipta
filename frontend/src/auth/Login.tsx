import { useState } from 'react'
import { useAuth } from '../contexts/AuthContext'
import { useNavigate } from 'react-router-dom'

function Login() {
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [message, setMessage] = useState({})
  const { setToken } = useAuth()
  const navigate = useNavigate()


  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      const res = await fetch('http://localhost:3000/api/auth/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password }),
      })

      if (!res.ok) {
        const errorText = await res.text()
        setMessage('Login failed: ' + errorText)
        return
      }

      const data = await res.json()
      if (data.token) {
        setToken(data.token)
        navigate('/dashboard')
      } else {
        setMessage('Login failed: no token in response')
      }
    } catch (err) {
      setMessage('Login error')
    }
  }


  return (
    <div className="min-h-screen bg-slate-50 flex items-center justify-center px-4">
      <div className="w-full max-w-md bg-white rounded-3xl shadow-2xl p-10 space-y-6 transition-all">
        <div className="text-center">
          <h1 className="text-3xl font-extrabold text-gray-900">🔐 Eclipta Login</h1>
          <p className="text-gray-500 text-sm mt-2">Authenticate to continue</p>
        </div>

        <form onSubmit={handleLogin} className="space-y-5">
          <div>
            <label className="block text-sm text-gray-600 mb-1">Username</label>
            <input
              type="text"
              value={username}
              onChange={e => setUsername(e.target.value)}
              className="w-full px-4 py-3 border border-gray-300 rounded-2xl bg-gray-50 focus:ring-2 focus:ring-indigo-500 focus:outline-none"
              placeholder="e.g., root"
            />
          </div>

          <div>
            <label className="block text-sm text-gray-600 mb-1">Password</label>
            <input
              type="password"
              value={password}
              onChange={e => setPassword(e.target.value)}
              className="w-full px-4 py-3 border border-gray-300 rounded-2xl bg-gray-50 focus:ring-2 focus:ring-indigo-500 focus:outline-none"
              placeholder="your system password"
            />
          </div>

          <button
            type="submit"
            className="w-full bg-indigo-600 text-white py-3 rounded-2xl font-semibold hover:bg-indigo-700 transition"
          >
            Sign In
          </button>
        </form>
      </div>
    </div>
  )
}

export default Login
