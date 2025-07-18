import { createContext, useContext, useState } from 'react'
import { useNavigate } from 'react-router-dom'

type AuthContextType = {
  token: string | null
  isAuthenticated: boolean
  setToken: (token: string | null) => void
  logout: () => void
}

const AuthContext = createContext<AuthContextType | undefined>(undefined)

export function AuthProvider({ children }: any) {
  const [token, setTokenState] = useState<string | null>(localStorage.getItem('token'));
  const navigate = useNavigate();

  const setToken = (newToken: string | null) => {
    setTokenState(newToken);
    if (newToken) {
      localStorage.setItem('token', newToken);
    } else {
      localStorage.removeItem('token');
    }
  };

  const logout = () => {
    setToken(null);
    navigate('/auth');
  };

  const isAuthenticated = token !== null;

  return (
    <AuthContext.Provider value={{ token, isAuthenticated, setToken, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext)
  if (!context) {
    throw new Error('useAuth must be used within AuthProvider')
  }
  return context
}
