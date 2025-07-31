import { Link } from 'react-router-dom'
import { Home, ArrowLeft } from 'lucide-react'
import { Button } from '@/components/ui/Button'

export default function NotFoundPage() {
  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50">
      <div className="max-w-md w-full text-center">
        {/* 404 Illustration */}
        <div className="mb-8">
          <div className="text-9xl font-bold text-primary-600 mb-4">404</div>
          <div className="w-24 h-1 bg-primary-600 mx-auto rounded-full"></div>
        </div>

        {/* Content */}
        <h1 className="text-3xl font-bold text-gray-900 mb-4">
          Page Not Found
        </h1>
        <p className="text-gray-600 mb-8">
          Sorry, we couldn't find the page you're looking for. 
          It might have been moved, deleted, or you entered the wrong URL.
        </p>

        {/* Actions */}
        <div className="flex flex-col sm:flex-row gap-4 justify-center">
          <Button
            variant="primary"
            leftIcon={<Home className="w-4 h-4" />}
            onClick={() => window.location.href = '/'}
          >
            Go Home
          </Button>
          <Button
            variant="secondary"
            leftIcon={<ArrowLeft className="w-4 h-4" />}
            onClick={() => window.history.back()}
          >
            Go Back
          </Button>
        </div>

        {/* Help Links */}
        <div className="mt-12 pt-8 border-t border-gray-200">
          <p className="text-sm text-gray-500 mb-4">
            Need help? Try these popular pages:
          </p>
          <div className="flex flex-wrap justify-center gap-4 text-sm">
            <Link
              to="/"
              className="text-primary-600 hover:text-primary-500 hover:underline"
            >
              Home
            </Link>
            <Link
              to="/raffles"
              className="text-primary-600 hover:text-primary-500 hover:underline"
            >
              Browse Raffles
            </Link>
            <Link
              to="/contact"
              className="text-primary-600 hover:text-primary-500 hover:underline"
            >
              Contact Support
            </Link>
          </div>
        </div>
      </div>
    </div>
  )
}