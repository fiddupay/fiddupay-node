export const Label = ({ children, className = '', ...props }: any) => (
  <label className={`block text-sm font-medium ${className}`} {...props}>{children}</label>
)
