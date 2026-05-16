#include "worldengine_c_api.h"

#include <memory>
#include <string>
#include <vector>

#include <worldengine/plates.h>

struct we_world
{
   std::shared_ptr<WorldEngine::World> world;
   std::vector<uint8_t>                ocean;
   std::vector<uint32_t>               biome;
};

static const float* float_data_or_null(const boost::multi_array<float, 2>& data)
{
   return data.num_elements() == 0 ? nullptr : data.data();
}

static WorldEngine::StepType step_type(int32_t step)
{
   switch (step)
   {
   case WE_STEP_PLATES: return WorldEngine::StepType::Plates;
   case WE_STEP_PRECIPITATIONS: return WorldEngine::StepType::Precipitations;
   case WE_STEP_FULL:
   default: return WorldEngine::StepType::Full;
   }
}

we_world* we_world_generate(const we_world_params* params)
{
   if (params == nullptr || params->width == 0 || params->height == 0)
   {
      return nullptr;
   }

   try
   {
      const std::string name =
         params->name == nullptr ? std::string("world") : std::string(params->name);
      auto* handle = new we_world();
      handle->world = WorldEngine::WorldGen(name,
                                            params->width,
                                            params->height,
                                            params->seed,
                                            WorldEngine::DEFAULT_TEMPS,
                                            WorldEngine::DEFAULT_HUMIDS,
                                            WorldEngine::DEFAULT_GAMMA_CURVE,
                                            WorldEngine::DEFAULT_CURVE_OFFSET,
                                            params->num_plates,
                                            params->ocean_level,
                                            WorldEngine::Step::step(step_type(params->step)),
                                            params->fade_borders);
      if (!handle->world)
      {
         delete handle;
         return nullptr;
      }
      return handle;
   }
   catch (...)
   {
      return nullptr;
   }
}

void we_world_free(we_world* world)
{
   delete world;
}

uint32_t we_world_width(const we_world* world)
{
   return world == nullptr ? 0 : world->world->width();
}

uint32_t we_world_height(const we_world* world)
{
   return world == nullptr ? 0 : world->world->height();
}

uint32_t we_world_seed(const we_world* world)
{
   return world == nullptr ? 0 : world->world->seed();
}

size_t we_world_len(const we_world* world)
{
   return world == nullptr ? 0 : static_cast<size_t>(world->world->width()) *
                                  static_cast<size_t>(world->world->height());
}

bool we_world_has_biome(const we_world* world)
{
   return world != nullptr && world->world->HasBiome();
}

bool we_world_has_humidity(const we_world* world)
{
   return world != nullptr && world->world->HasHumidity();
}

bool we_world_has_icecap(const we_world* world)
{
   return world != nullptr && world->world->HasIcecap();
}

bool we_world_has_irrigation(const we_world* world)
{
   return world != nullptr && world->world->HasIrrigation();
}

bool we_world_has_lakemap(const we_world* world)
{
   return world != nullptr && world->world->HasLakemap();
}

bool we_world_has_permeability(const we_world* world)
{
   return world != nullptr && world->world->HasPermeability();
}

bool we_world_has_precipitation(const we_world* world)
{
   return world != nullptr && world->world->HasPrecipitations();
}

bool we_world_has_rivermap(const we_world* world)
{
   return world != nullptr && world->world->HasRivermap();
}

bool we_world_has_temperature(const we_world* world)
{
   return world != nullptr && world->world->HasTemperature();
}

bool we_world_has_watermap(const we_world* world)
{
   return world != nullptr && world->world->HasWatermap();
}

const float* we_world_elevation(const we_world* world)
{
   return world == nullptr ? nullptr : float_data_or_null(world->world->GetElevationData());
}

const uint8_t* we_world_ocean(we_world* world)
{
   if (world == nullptr)
   {
      return nullptr;
   }

   const auto& ocean = world->world->GetOceanData();
   if (ocean.num_elements() == 0)
   {
      return nullptr;
   }

   world->ocean.resize(ocean.num_elements());
   for (size_t i = 0; i < ocean.num_elements(); ++i)
   {
      world->ocean[i] = ocean.data()[i] ? 1u : 0u;
   }
   return world->ocean.data();
}

const uint16_t* we_world_plates(const we_world* world)
{
   if (world == nullptr)
   {
      return nullptr;
   }
   const auto& plates = world->world->GetPlateData();
   return plates.num_elements() == 0 ? nullptr : plates.data();
}

const uint32_t* we_world_biome(we_world* world)
{
   if (world == nullptr || !world->world->HasBiome())
   {
      return nullptr;
   }

   const auto& biome = world->world->GetBiomeData();
   world->biome.resize(biome.num_elements());
   for (size_t i = 0; i < biome.num_elements(); ++i)
   {
      world->biome[i] = static_cast<uint32_t>(biome.data()[i]);
   }
   return world->biome.data();
}

const float* we_world_humidity(const we_world* world)
{
   return world == nullptr || !world->world->HasHumidity()
             ? nullptr
             : float_data_or_null(world->world->GetHumidityData());
}

const float* we_world_icecap(const we_world* world)
{
   return world == nullptr || !world->world->HasIcecap()
             ? nullptr
             : float_data_or_null(world->world->GetIcecapData());
}

const float* we_world_irrigation(const we_world* world)
{
   return world == nullptr || !world->world->HasIrrigation()
             ? nullptr
             : float_data_or_null(world->world->GetIrrigationData());
}

const float* we_world_lakemap(const we_world* world)
{
   return world == nullptr || !world->world->HasLakemap()
             ? nullptr
             : float_data_or_null(world->world->GetLakeMapData());
}

const float* we_world_permeability(const we_world* world)
{
   return world == nullptr || !world->world->HasPermeability()
             ? nullptr
             : float_data_or_null(world->world->GetPermeabilityData());
}

const float* we_world_precipitation(const we_world* world)
{
   return world == nullptr || !world->world->HasPrecipitations()
             ? nullptr
             : float_data_or_null(world->world->GetPrecipitationData());
}

const float* we_world_rivermap(const we_world* world)
{
   return world == nullptr || !world->world->HasRivermap()
             ? nullptr
             : float_data_or_null(world->world->GetRiverMapData());
}

const float* we_world_sea_depth(const we_world* world)
{
   return world == nullptr ? nullptr : float_data_or_null(world->world->GetSeaDepthData());
}

const float* we_world_temperature(const we_world* world)
{
   return world == nullptr || !world->world->HasTemperature()
             ? nullptr
             : float_data_or_null(world->world->GetTemperatureData());
}

const float* we_world_watermap(const we_world* world)
{
   return world == nullptr || !world->world->HasWatermap()
             ? nullptr
             : float_data_or_null(world->world->GetWaterMapData());
}

