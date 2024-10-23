import { FormControl, FormHelperText, FormLabel } from "@mui/joy";
import { ValidationError } from "@tanstack/react-form";
import { ReactNode } from "react";

export function FormField({
  errors,
  children,
  label,
}: {
  errors: ValidationError[];
  children: ReactNode;
  label: string;
}) {
  return (
    <FormControl error={errors.length > 0}>
      <FormLabel>{label}</FormLabel>
      {children}

      {errors.map((e, idx) => (
        <FormHelperText key={idx}>{e}</FormHelperText>
      ))}
    </FormControl>
  );
}
