export const Select = ({ children, className = '', ...props }: any) => (
  <select className={`border rounded px-3 py-2 ${className}`} {...props}>{children}</select>
)

export const SelectContent = ({ children }: { children: React.ReactNode }) => <>{children}</>
export const SelectItem = ({ children, ...props }: any) => <option {...props}>{children}</option>
export const SelectTrigger = ({ children, className = '' }: { children: React.ReactNode; className?: string }) => <div className={className}>{children}</div>
export const SelectValue = ({ placeholder }: { placeholder?: string }) => <span>{placeholder}</span>
