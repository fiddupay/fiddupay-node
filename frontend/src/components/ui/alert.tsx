export const Alert = ({ children, className = '', variant, ...props }: any) => (
  <div className={`p-4 border rounded ${className}`} {...props}>{children}</div>
)

export const AlertDescription = ({ children, className = '' }: { children: React.ReactNode; className?: string }) => (
  <div className={className}>{children}</div>
)
