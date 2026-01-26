export const Button = ({ children, className = '', variant, ...props }: any) => (
  <button className={`px-4 py-2 rounded ${className}`} {...props}>{children}</button>
)
