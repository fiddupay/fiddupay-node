import React, { useEffect } from 'react'
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom'
import { useAuthStore } from '@/stores/authStore'
import { ToastProvider } from '@/contexts/ToastContext'
import { LoadingProvider } from '@/contexts/LoadingContext'
import Layout from '@/components/Layout'
import AppLayout from '@/components/layout/AppLayout'
import LoginPage from '@/pages/auth/LoginPage'
import RegisterPage from '@/pages/auth/RegisterPage'
import ForgotPasswordPage from '@/pages/auth/ForgotPasswordPage'
import HomePage from '@/pages/HomePage'
import AboutPage from '@/pages/AboutPage'
import FeaturesPage from '@/pages/FeaturesPage'
import PricingPage from '@/pages/PricingPage'
import DocsPage from '@/pages/DocsPage'
import ContactPage from '@/pages/ContactPage'
import TermsPage from '@/pages/TermsPage'
import PrivacyPage from '@/pages/PrivacyPage'
import CareersPage from '@/pages/CareersPage'
import BlogPage from '@/pages/BlogPage'
import StatusPage from '@/pages/StatusPage'
import SecurityPage from '@/pages/SecurityPage'
import CompliancePage from '@/pages/CompliancePage'
import CookiesPage from '@/pages/CookiesPage'
import ColorTestPage from '@/pages/ColorTestPage'
import '@/styles/globals.css'

// Lazy load pages for better performance
const DashboardPage = React.lazy(() => import('@/pages/DashboardPage'))
const PaymentsPage = React.lazy(() => import('@/pages/PaymentsPage'))
const WalletsPage = React.lazy(() => import('@/pages/WalletsPage'))

const App: React.FC = () => {
  const { loadUser, loading } = useAuthStore()

  useEffect(() => {
    loadUser()
  }, [loadUser])

  if (loading) {
    return (
      <div style={{
        minHeight: '100vh',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center'
      }}>
        <div className="animate-spin" style={{
          width: '2rem',
          height: '2rem',
          border: '2px solid #e5e7eb',
          borderTop: '2px solid #1e40af',
          borderRadius: '50%'
        }} />
      </div>
    )
  }

  return (
    <ToastProvider>
      <LoadingProvider>
        <Router>
          <Routes>
            {/* Public routes */}
            <Route path="/" element={<Layout><HomePage /></Layout>} />
            <Route path="/about" element={<Layout><AboutPage /></Layout>} />
            <Route path="/features" element={<Layout><FeaturesPage /></Layout>} />
            <Route path="/pricing" element={<Layout><PricingPage /></Layout>} />
            <Route path="/docs" element={<Layout><DocsPage /></Layout>} />
            <Route path="/contact" element={<Layout><ContactPage /></Layout>} />
            <Route path="/terms" element={<Layout><TermsPage /></Layout>} />
            <Route path="/privacy" element={<Layout><PrivacyPage /></Layout>} />
            <Route path="/careers" element={<Layout><CareersPage /></Layout>} />
            <Route path="/blog" element={<Layout><BlogPage /></Layout>} />
            <Route path="/status" element={<Layout><StatusPage /></Layout>} />
            <Route path="/security" element={<Layout><SecurityPage /></Layout>} />
            <Route path="/compliance" element={<Layout><CompliancePage /></Layout>} />
            <Route path="/cookies" element={<Layout><CookiesPage /></Layout>} />
            <Route path="/color-test" element={<Layout><ColorTestPage /></Layout>} />
            
            {/* Auth routes */}
            <Route path="/login" element={<Layout><LoginPage /></Layout>} />
            <Route path="/register" element={<Layout><RegisterPage /></Layout>} />
            <Route path="/forgot-password" element={<Layout><ForgotPasswordPage /></Layout>} />
            
            {/* Protected routes */}
            <Route path="/app" element={<AppLayout />}>
              <Route index element={<Navigate to="/app/dashboard" replace />} />
              <Route 
                path="dashboard" 
                element={
                  <React.Suspense fallback={<div>Loading...</div>}>
                    <DashboardPage />
                  </React.Suspense>
                } 
              />
              <Route 
                path="payments" 
                element={
                  <React.Suspense fallback={<div>Loading...</div>}>
                    <PaymentsPage />
                  </React.Suspense>
                } 
              />
              <Route 
                path="wallets" 
                element={
                  <React.Suspense fallback={<div>Loading...</div>}>
                    <WalletsPage />
                  </React.Suspense>
                } 
              />
            </Route>
            
            {/* Catch all route */}
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </Router>
      </LoadingProvider>
    </ToastProvider>
  )
}

export default App
