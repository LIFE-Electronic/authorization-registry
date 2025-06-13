import {
  FormControl,
  FormControlProps,
  FormHelperText,
  FormLabel,
} from "@mui/joy";
import { ValidationError } from "@tanstack/react-form";
import { ReactNode } from "react";

export function FormField({
  errors,
  children,
  label,
  formControlProps,
}: {
  errors: ValidationError[];
  children: ReactNode;
  label: string;
  formControlProps?: FormControlProps;
}) {
  return (
    <FormControl {...formControlProps} error={errors.length > 0}>
      <FormLabel>{label}</FormLabel>
      {children}

      {errors.map((e, idx) => (
        <FormHelperText key={idx}>{e as string}</FormHelperText>
      ))}
    </FormControl>
  );
}
