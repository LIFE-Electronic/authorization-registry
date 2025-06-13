import { Typography, TypographyProps } from "@mui/joy";

export function Caption(props: TypographyProps) {
  return (
    <Typography
      fontSize="11px"
      textTransform="uppercase"
      fontWeight={400}
      textColor="neutral.500"
      letterSpacing="0px"
      {...props}
    ></Typography>
  );
}

export function Subtitle2(props: TypographyProps) {
  return (
    <Typography
      fontSize="16px"
      fontWeight={700}
      textColor="neutral.900"
      fontFamily="Inter Variable"
      {...props}
    ></Typography>
  );
}
