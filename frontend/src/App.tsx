import { Routes, Route, Navigate } from 'react-router-dom'
import { useAuth } from './contexts/AuthContext'
import Login from './auth/Login'
import Dashboard from './pages/dashboard'
import Agents from './pages/Agents'

function App() {
  const { isAuthenticated } = useAuth()

  return (
    <Routes>
      <Route path="/auth" element={<Login />} />
      <Route
        path="/dashboard"
        element={isAuthenticated ? <Dashboard /> : <Navigate to="/auth" />}
      />
      <Route
        path="/agents"
        element={isAuthenticated ? <Agents /> : <Navigate to="/auth" />}
      />
      <Route path="*" element={<Navigate to="/dashboard" />} />
    </Routes>
  )
}

export default App
