import React from 'react';
import styles from './Skeleton.module.css';

interface SkeletonProps {
    width?: string | number;
    height?: string | number;
    borderRadius?: string | number;
    className?: string;
    variant?: 'text' | 'rect' | 'circle';
    style?: React.CSSProperties;
}

const Skeleton: React.FC<SkeletonProps> = ({
    width,
    height,
    borderRadius,
    className = '',
    variant = 'rect',
    style: customStyle = {}
}) => {
    const style: React.CSSProperties = {
        width,
        height,
        borderRadius: variant === 'circle' ? '50%' : borderRadius,
        ...customStyle
    };

    return (
        <div 
            className={`${styles.skeleton} ${styles[variant]} ${className}`} 
            style={style}
        />
    );
};

export default Skeleton;
