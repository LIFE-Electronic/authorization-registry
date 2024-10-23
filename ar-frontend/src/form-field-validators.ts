export const required = {
  onSubmit: <T>({ value }: { value: T }) => {
    if (!value || (Array.isArray(value) && value.length === 0)) {
      return "Field is required";
    }
  },
};
