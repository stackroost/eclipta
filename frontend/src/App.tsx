import { BrowserRouter as Router, Routes, Route } from 'react-router-dom'
import Login from './auth/Login'

function App() {
  return (
    <Router>
      <Routes>
        <Route path="/auth" element={<Login />} />
        {/* Add more routes as needed */}
        <Route path="*" element={<div className="p-10  text-xl">404 Not Found</div>} />
      </Routes>
    </Router>
  )
}

export default App
