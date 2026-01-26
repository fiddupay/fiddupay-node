export const Badge = ({ children, className = '', variant, ...props }: any) => (
  <span className={`inline-block px-2 py-1 text-xs rounded ${className}`} {...props}>{children}</span>
)
