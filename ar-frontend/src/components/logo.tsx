import { Typography, Box } from "@mui/joy";
import { LogoPurple } from "./logo-purple";

export function Logo({ admin }: { admin: boolean }) {
  return (
    <>
      <LogoPurple />
      {/* 
      <Typography
        textColor="neutral.600"
        fontSize={10}
        fontWeight={600}
        letterSpacing="-0.5px"
      >
        Authorization registry {admin ? " (admin)" : ""}
      </Typography> */}
    </>
  );
}
