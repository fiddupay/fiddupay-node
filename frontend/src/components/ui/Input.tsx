import React from 'react'
import styles from './Input.module.css'

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string
  error?: string
  helperText?: string
}

const Input: React.FC<InputProps> = ({
  label,
  error,
  helperText,
  className = '',
  id,
  ...props
}) => {
  const inputId = id || `input-${Math.random().toString(36).substr(2, 9)}`
  
  const inputClass = [
    styles.input,
    error && styles.inputError,
    className,
  ].filter(Boolean).join(' ')

  return (
    <div className={styles.container}>
      {label && (
        <label htmlFor={inputId} className={styles.label}>
          {label}
        </label>
      )}
      
      <input
        id={inputId}
        className={inputClass}
        {...props}
      />
      
      {error && (
        <p className={styles.error}>{error}</p>
      )}
      
      {helperText && !error && (
        <p className={styles.helperText}>{helperText}</p>
      )}
    </div>
  )
}

export default Input
