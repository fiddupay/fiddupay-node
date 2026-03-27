import { Customer, STATUS_STYLES } from "./types";

export const getStatusStyle = (status: string) => {
  return STATUS_STYLES[status] || STATUS_STYLES.active;
};

export const getInitials = (customer: Customer) => {
  const f = customer.first_name?.[0] || "";
  const l = customer.last_name?.[0] || "";
  if (!f && !l) return customer.external_id.substring(0, 2).toUpperCase();
  return (f + l).toUpperCase();
};
