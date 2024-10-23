import { Typography, Box } from "@mui/joy";
import { LogoZwart } from "./logo-zwart";

export function Logo() {
  return (
    <Box>
      <LogoZwart />

      <Typography
        textColor="neutral.600"
        fontSize={10}
        fontWeight={600}
        letterSpacing="-0.5px"
      >
        Authorization registry
      </Typography>
    </Box>
  );
}
