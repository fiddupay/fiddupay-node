import React from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { Link, Navigate } from 'react-router-dom'
import { useAuthStore } from '@/stores/authStore'
import Button from '@/components/ui/Button'
import Input from '@/components/ui/Input'
import styles from './LoginPage.module.css'

const loginSchema = z.object({
  email: z.string().email('Invalid email address'),
  password: z.string().min(6, 'Password must be at least 6 characters'),
  two_factor_code: z.string().optional(),
})

type LoginFormData = z.infer<typeof loginSchema>

const LoginPage: React.FC = () => {
  const { login, loading, error, isAuthenticated } = useAuthStore()
  
  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<LoginFormData>({
    resolver: zodResolver(loginSchema),
  })

  if (isAuthenticated) {
    return <Navigate to="/app/dashboard" replace />
  }

  const onSubmit = async (data: LoginFormData) => {
    try {
      await login(data)
    } catch (error) {
      // Error is handled by the store
    }
  }

  return (
    <div className={styles.container}>
      <div className={styles.card}>
        <div className={styles.header}>
          <h1 className={styles.title}>PayFlow</h1>
          <p className={styles.subtitle}>Sign in to your merchant account</p>
        </div>

        <form onSubmit={handleSubmit(onSubmit)} className={styles.form}>
          {error && (
            <div className={styles.errorAlert}>
              {error}
            </div>
          )}

          <Input
            label="Email address"
            type="email"
            {...register('email')}
            error={errors.email?.message}
            placeholder="Enter your email"
          />

          <Input
            label="Password"
            type="password"
            {...register('password')}
            error={errors.password?.message}
            placeholder="Enter your password"
          />

          <Input
            label="2FA Code (if enabled)"
            type="text"
            {...register('two_factor_code')}
            error={errors.two_factor_code?.message}
            placeholder="Enter 6-digit code"
          />

          <Button
            type="submit"
            loading={loading}
            className={styles.submitButton}
          >
            Sign in
          </Button>
        </form>

        <div className={styles.footer}>
          <p>
            Don't have an account?{' '}
            <Link to="/register" className={styles.link}>
              Sign up
            </Link>
          </p>
        </div>
      </div>
    </div>
  )
}

export default LoginPage
