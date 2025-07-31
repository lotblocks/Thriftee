import { Header } from './Header'
import { Footer } from './Footer'
import { Sidebar } from './Sidebar'
import { useAuth } from '@/contexts/AuthContext'

interface LayoutProps {
  children: React.ReactNode
}

export function Layout({ children }: LayoutProps) {
  const { user } = useAuth()

  return (
    <div className="min-h-screen bg-gray-50">
      <Header />
      
      <div className="flex">
        {user && <Sidebar />}
        
        <main className={cn(
          'flex-1 min-h-screen',
          user ? 'lg:ml-64' : ''
        )}>
          <div className="container-padding section-padding">
            {children}
          </div>
        </main>
      </div>
      
      <Footer />
    </div>
  )
}

// Re-export components for easier imports
export { Header } from './Header'
export { Footer } from './Footer'
export { Sidebar } from './Sidebar'