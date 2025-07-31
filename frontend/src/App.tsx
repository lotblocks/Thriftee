import { Routes, Route, Navigate } from 'react-router-dom'
import { useEffect } from 'react'
import { useAuth } from './contexts/AuthContext'
import { Layout } from './components/Layout'
import MobileLayout from './components/Mobile/MobileLayout'
import { LoadingSpinner } from './components/ui/LoadingSpinner'
import { useResponsive } from './hooks/useResponsive'
import { initializeMobileUtils } from './utils/mobileUtils'
import { pwaService } from './services/pwaService'

// Pages
import HomePage from './pages/HomePage'
import RafflesPage from './pages/Raffles/RafflesPage'
import RaffleDetailPage from './pages/Raffles/RaffleDetailPage'
import DashboardPage from './pages/Dashboard/DashboardPage'
import ProfilePage from './pages/Profile/ProfilePage'
import WalletPage from './pages/Wallet/WalletPage'
import LoginPage from './pages/Auth/LoginPage'
import RegisterPage from './pages/Auth/RegisterPage'
import NotFoundPage from './pages/NotFoundPage'

// Protected Route Component
function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { user, isLoading } = useAuth()

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <LoadingSpinner size="lg" />
      </div>
    )
  }

  if (!user) {
    return <Navigate to="/login" replace />
  }

  return <>{children}</>
}

// Public Route Component (redirect to dashboard if authenticated)
function PublicRoute({ children }: { children: React.ReactNode }) {
  const { user, isLoading } = useAuth()

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <LoadingSpinner size="lg" />
      </div>
    )
  }

  if (user) {
    return <Navigate to="/dashboard" replace />
  }

  return <>{children}</>
}

function App() {
  const { isMobile } = useResponsive()

  // Initialize mobile optimizations and PWA
  useEffect(() => {
    initializeMobileUtils()
    
    // Register service worker for PWA functionality
    pwaService.registerServiceWorker()
  }, [])

  // Choose layout based on device type
  const AppLayout = isMobile ? MobileLayout : Layout

  return (
    <div className="min-h-screen bg-gray-50">
      <Routes>
        {/* Public routes */}
        <Route path="/" element={<AppLayout><HomePage /></AppLayout>} />
        <Route path="/raffles" element={<AppLayout><RafflesPage /></AppLayout>} />
        <Route path="/raffles/:id" element={<AppLayout showBottomNav={false}><RaffleDetailPage /></AppLayout>} />
        
        {/* Auth routes (redirect if already authenticated) */}
        <Route 
          path="/login" 
          element={
            <PublicRoute>
              {isMobile ? (
                <MobileLayout>
                  <LoginPage />
                </MobileLayout>
              ) : (
                <LoginPage />
              )}
            </PublicRoute>
          } 
        />
        <Route 
          path="/register" 
          element={
            <PublicRoute>
              {isMobile ? (
                <MobileLayout>
                  <RegisterPage />
                </MobileLayout>
              ) : (
                <RegisterPage />
              )}
            </PublicRoute>
          } 
        />
        
        {/* Protected routes */}
        <Route 
          path="/dashboard" 
          element={
            <ProtectedRoute>
              <AppLayout>
                <DashboardPage />
              </AppLayout>
            </ProtectedRoute>
          } 
        />
        <Route 
          path="/profile" 
          element={
            <ProtectedRoute>
              <AppLayout>
                <ProfilePage />
              </AppLayout>
            </ProtectedRoute>
          } 
        />
        <Route 
          path="/wallet" 
          element={
            <ProtectedRoute>
              <AppLayout>
                <WalletPage />
              </AppLayout>
            </ProtectedRoute>
          } 
        />
        
        {/* 404 route */}
        <Route path="*" element={<AppLayout><NotFoundPage /></AppLayout>} />
      </Routes>
    </div>
  )
}

export default App