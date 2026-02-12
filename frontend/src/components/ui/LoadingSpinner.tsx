import React from 'react'
import styles from './LoadingSpinner.module.css'

interface LoadingSpinnerProps {
    size?: 'small' | 'medium' | 'large'
    color?: 'primary' | 'white'
    className?: string
}

const LoadingSpinner: React.FC<LoadingSpinnerProps> = ({
    size = 'medium',
    color = 'primary',
    className = ''
}) => {
    return (
        <div className={`${styles.spinner} ${styles[size]} ${styles[color]} ${className}`} />
    )
}

export default LoadingSpinner
