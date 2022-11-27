// Copyright Epic Games, Inc. All Rights Reserved.

#pragma once

#include "Templates/Tuple.h"
#include "Containers/Map.h"
#include "Misc/TVariant.h"
#include "MovieSceneCommonHelpers.h"
#include "EntitySystem/MovieSceneEntitySystemLinkerExtension.h"
#include "EntitySystem/MovieScenePropertySystemTypes.h"

namespace UE
{
namespace MovieScene
{

template<typename PropertyType> struct TPropertyValueStorage;

/** Type-safe index that defines a unique index of an initial value within a TPropertyValueStorage instance
 *  This struct is added as a component to all entities with initial values inside non-interrogated linkers */
struct FInitialValueIndex
{
	uint16 Index;
};

/** Base interface for all initial value storage types */
struct IPropertyValueStorage
{
	virtual ~IPropertyValueStorage(){}

	/** Reset all the initial values for the specified indices */
	virtual void Reset(TArrayView<const FInitialValueIndex> Indices) = 0;
};

/**
 * Container that houses initial values for any properties animated through FEntityManager
 * Each type is stored in its own container, organized by the initial value component ID
 * The cache is stored as a singleton (through GetSharedInitialValues()) and added as an
 * extension to UMovieSceneEntitySystemLinker where it is active.
 */
struct FInitialValueCache
{
	/**
	 * Retrieve the extension ID for this structure when added to a UMovieSceneEntitySystemLinker
	 */
	MOVIESCENE_API static TEntitySystemLinkerExtensionID<FInitialValueCache> GetExtensionID();


	/**
	 * Retrieve a container used for sharing initial values between linkers.
	 * @note: The instance is not referenced internally, so will be destroyed when all
	 * externally owned shared pointers are destroyed.
	 */
	MOVIESCENE_API static TSharedPtr<FInitialValueCache> GetGlobalInitialValues();

public:

	/**
	 * Reset all the initial values that relate to the specified type and indices
	 *
	 * @param InitialValueType      The ComponentTypeID for the initial value that the indices relate to
	 * @param InitialValueIndices   An array containing all the indices to remove
	 */
	void Reset(FComponentTypeID InitialValueType, TArrayView<const FInitialValueIndex> InitialValueIndices)
	{
		if (TUniquePtr<IPropertyValueStorage>* Storage = StorageByComponent.Find(InitialValueType))
		{
			Storage->Get()->Reset(InitialValueIndices);
		}
	}

	/**
	 * Retrieve the initial value storage for a given initial value type, creating it if necessary.
	 * @note: Care should be taken to ensure that the template parameter matches TPropertyComponents::PropertyType,
	 * not TPropertyComponents::OperationalType.
	 *
	 * @param InitialValueType      The ComponentTypeID for the initial value
	 * @return Property storage for the initial values
	 */
	template<typename PropertyType>
	TPropertyValueStorage<PropertyType>* GetStorage(FComponentTypeID InitialValueType)
	{
		if (TPropertyValueStorage<PropertyType>* Storage = FindStorage<PropertyType>(InitialValueType))
		{
			return Storage;
		}

		TPropertyValueStorage<PropertyType>* NewStorage = new TPropertyValueStorage<PropertyType>();
		StorageByComponent.Add(InitialValueType, TUniquePtr<IPropertyValueStorage>(NewStorage));
		return NewStorage;
	}


	/**
	 * Retrieve the initial value storage for a given initial value type.
	 * @note: Care should be taken to ensure that the template parameter matches TPropertyComponents::PropertyType,
	 * not TPropertyComponents::OperationalType.
	 *
	 * @param InitialValueType      The ComponentTypeID for the initial value
	 * @return Property storage for the initial values or nullptr if none exists
	 */
	template<typename PropertyType>
	TPropertyValueStorage<PropertyType>* FindStorage(FComponentTypeID InitialValueType)
	{
		if (TUniquePtr<IPropertyValueStorage>* Existing = StorageByComponent.Find(InitialValueType))
		{
			// If the ptr exists, it cannot be null
			check(Existing->IsValid());
			return static_cast<TPropertyValueStorage<PropertyType>*>(Existing->Get());
		}

		return nullptr;
	}

private:

	TMap<FComponentTypeID, TUniquePtr<IPropertyValueStorage>> StorageByComponent;
};

/**
 * Templated storage for any initial value type, templated on TPropertyComponents::PropertyType for correct retrieval of resolved properties
 * Initial values are stored as a sparse array, with stable indices that uniquely identify the value.
 * A look-up-table exists for finding indices based on an object and resolved property.
 */
template<typename PropertyType>
struct TPropertyValueStorage : IPropertyValueStorage
{
	/**
	 * Reset all the initial values that relate to the specified indices
	 *
	 * @param Indices    A array specifying all of the indices to remove. Such indices will be invalid after this function call returns.
	 */
	virtual void Reset(TArrayView<const FInitialValueIndex> Indices) override
	{
		for (FInitialValueIndex Index : Indices)
		{
			PropertyValues.RemoveAt(Index.Index);
		}
		bLUTContainsInvalidEntries = true;
	}


	/**
	 * Retrieve the cached value for the specified object and fast property ptr offset, returning both the value and its corresponding index.
	 * @note: if the value is already cached, the existing value is returned
	 *
	 * @param BoundObject            The object instance to cache the property from
	 * @param ResolvedPropertyOffset The byte offset from BoundObject that defines the address of the property
	 * @return A tuple containing the cached value and its index
	 */
	TPair<FInitialValueIndex, PropertyType> CacheInitialValue(UObject* BoundObject, uint16 ResolvedPropertyOffset)
	{
		FKeyType Key{ FObjectKey(BoundObject), FPropertyKey(TInPlaceType<uint16>(), ResolvedPropertyOffset) };

		if (TOptional<FInitialValueIndex> ExistingIndex = FindPropertyIndex(Key))
		{
			FInitialValueIndex ExistingValue = ExistingIndex.GetValue();
			return MakeTuple(ExistingValue, PropertyValues[ExistingValue.Index]);
		}

		PropertyType CachedValue = *reinterpret_cast<const PropertyType*>(reinterpret_cast<const uint8*>(static_cast<const void*>(BoundObject)) + ResolvedPropertyOffset);

		const int32 NewIndex = PropertyValues.Add(CachedValue);
		check(NewIndex < int32(uint16(0xFFFF)));

		const uint16 NarrowIndex = static_cast<uint16>(NewIndex);
		KeyToPropertyIndex.Add(Key, NarrowIndex);

		return MakeTuple(FInitialValueIndex{NarrowIndex}, CachedValue);
	}


	/**
	 * Retrieve the cached value for the specified object and a custom property accessor, returning both the value and its corresponding index.
	 * @note: if the value is already cached, the existing value is returned
	 *
	 * @param BoundObject            The object instance to cache the property from.
	 * @param Accessors              A type-erased view retrieved from the necessary ICustomPropertyRegistration::GetAccessors for the property type this cache relates to.
	 * @param AccessorIndex          The index into Accessors to use for resolving the property
	 * @return A tuple containing the cached value and its index
	 */
	TPair<FInitialValueIndex, PropertyType> CacheInitialValue(UObject* BoundObject, FCustomAccessorView Accessors, FCustomPropertyIndex AccessorIndex)
	{
		FKeyType Key{ FObjectKey(BoundObject), FPropertyKey(TInPlaceType<FCustomPropertyIndex>(), AccessorIndex) };

		if (TOptional<FInitialValueIndex> ExistingIndex = FindPropertyIndex(Key))
		{
			FInitialValueIndex ExistingValue = ExistingIndex.GetValue();
			return MakeTuple(ExistingValue, PropertyValues[ExistingValue.Index]);
		}

		const TCustomPropertyAccessor<PropertyType>& CustomAccessor = static_cast<const TCustomPropertyAccessor<PropertyType>&>(Accessors[AccessorIndex.Value]);
		PropertyType CachedValue = CustomAccessor.Functions.Getter(BoundObject);

		const int32 NewIndex = PropertyValues.Add(CachedValue);
		check(NewIndex < int32(uint16(0xFFFF)));

		const uint16 NarrowIndex = static_cast<uint16>(NewIndex);
		KeyToPropertyIndex.Add(Key, NarrowIndex);

		return MakeTuple(FInitialValueIndex{NarrowIndex}, CachedValue);
	}


	/**
	 * Retrieve the cached value for the specified object and a slow bindings instance, returning both the value and its corresponding index.
	 * @note: if the value is already cached, the existing value is returned
	 *
	 * @param BoundObject            The object instance to cache the property from.
	 * @param SlowBindings           Pointer to the track instance property bindings object used for retrieving the property value
	 * @return A tuple containing the cached value and its index
	 */
	TPair<FInitialValueIndex, PropertyType> CacheInitialValue(UObject* BoundObject, FTrackInstancePropertyBindings* SlowBindings)
	{
		FKeyType Key{ FObjectKey(BoundObject), FPropertyKey(TInPlaceType<FName>(), SlowBindings->GetPropertyPath())};

		if (TOptional<FInitialValueIndex> ExistingIndex = FindPropertyIndex(Key))
		{
			FInitialValueIndex ExistingValue = ExistingIndex.GetValue();
			return MakeTuple(ExistingValue, PropertyValues[ExistingValue.Index]);
		}

		PropertyType CachedValue = SlowBindings->GetCurrentValue<PropertyType>(*BoundObject);

		const int32 NewIndex = PropertyValues.Add(CachedValue);
		check(NewIndex < int32(uint16(0xFFFF)));

		const uint16 NarrowIndex = static_cast<uint16>(NewIndex);
		KeyToPropertyIndex.Add(Key, NarrowIndex);

		return MakeTuple(FInitialValueIndex{NarrowIndex}, CachedValue);
	}


	/**
	 * Find an initial value index given its object and fast ptr offset
	 */
	TOptional<FInitialValueIndex> FindPropertyIndex(UObject* BoundObject, uint16 ResolvedPropertyOffset)
	{
		CleanupStaleEntries();
		return FindPropertyIndex(FKeyType{ FObjectKey(BoundObject), FPropertyKey(TInPlaceType<uint16>(), ResolvedPropertyOffset) });
	}


	/**
	 * Find an initial value index given its object and custom accessor index
	 */
	TOptional<FInitialValueIndex> FindPropertyIndex(UObject* BoundObject, FCustomPropertyIndex AccessorIndex)
	{
		CleanupStaleEntries();
		return FindPropertyIndex(FKeyType{ FObjectKey(BoundObject), FPropertyKey(TInPlaceType<FCustomPropertyIndex>(), AccessorIndex) });
	}


	/**
	 * Find an initial value index given its object and property name.
	 * @note: Only properties cached using a FTrackInstancePropertyBindings instance will be retrieved using this method.
	 */
	TOptional<FInitialValueIndex> FindPropertyIndex(UObject* BoundObject, const FName& PropertyPath)
	{
		CleanupStaleEntries();
		return FindPropertyIndex(FKeyType{ FObjectKey(BoundObject), FPropertyKey(TInPlaceType<FName>(), PropertyPath) });
	}


	/**
	 * Find an initial value given its object and property name.
	 */
	const PropertyType* FindCachedValue(UObject* BoundObject, uint16 ResolvedPropertyOffset)
	{
		TOptional<FInitialValueIndex> Index = FindPropertyIndex(BoundObject, ResolvedPropertyOffset);
		return Index.IsSet() ? &PropertyValues[Index.GetValue().Index] : nullptr;
	}


	/**
	 * Find an initial value given its object and custom accessor index
	 */
	const PropertyType* FindCachedValue(UObject* BoundObject, FCustomPropertyIndex CustomIndex)
	{
		TOptional<FInitialValueIndex> Index = FindPropertyIndex(BoundObject, CustomIndex);
		return Index.IsSet() ? &PropertyValues[Index.GetValue().Index] : nullptr;
	}


	/**
	 * Find an initial value given its object and property name.
	 * @note: Only properties cached using a FTrackInstancePropertyBindings instance will be retrieved using this method.
	 */
	const PropertyType* FindCachedValue(UObject* BoundObject, const FName& PropertyPath)
	{
		TOptional<FInitialValueIndex> Index = FindPropertyIndex(BoundObject, PropertyPath);
		return Index.IsSet() ? &PropertyValues[Index.GetValue().Index] : nullptr;
	}

private:

	using FPropertyKey = TVariant<uint16, FCustomPropertyIndex, FName>;

	struct FKeyType
	{
		FObjectKey Object;
		FPropertyKey Property;

		friend uint32 GetTypeHash(const FKeyType& InKey)
		{
			// Hash only considers the _type_ of the resolved property
			// which defers the final comparison to the equality operator
			uint32 Hash = GetTypeHash(InKey.Object);
			Hash = HashCombine(Hash, InKey.Property.GetIndex());
			return Hash;
		}
		friend bool operator==(const FKeyType& A, const FKeyType& B)
		{
			if (A.Object != B.Object || A.Property.GetIndex() != B.Property.GetIndex())
			{
				return false;
			}
			switch (A.Property.GetIndex())
			{
				case 0: return A.Property.template Get<uint16>() == B.Property.template Get<uint16>();
				case 1: return A.Property.template Get<FCustomPropertyIndex>().Value == B.Property.template Get<FCustomPropertyIndex>().Value;
				case 2: return A.Property.template Get<FName>() == B.Property.template Get<FName>();
			}
			return true;
		}
	};

	FORCEINLINE void CleanupStaleEntries()
	{
		if (!bLUTContainsInvalidEntries)
		{
			return;
		}
		for (auto It = KeyToPropertyIndex.CreateIterator(); It; ++It)
		{
			if (!PropertyValues.IsValidIndex(It.Value()))
			{
				It.RemoveCurrent();
			}
		}
		PropertyValues.Shrink();
		bLUTContainsInvalidEntries = false;
	}

	TOptional<FInitialValueIndex> FindPropertyIndex(const FKeyType& InKey)
	{
		CleanupStaleEntries();

		const uint16* Index = KeyToPropertyIndex.Find(InKey);
		return Index ? TOptional<FInitialValueIndex>(FInitialValueIndex{*Index}) : TOptional<FInitialValueIndex>();
	}

	/** Sparse array containing all cached property values */
	TSparseArray<PropertyType> PropertyValues;
	/** LUT from object+property to its index. May contain stale values if bLUTContainsInvalidEntries is true */
	TMap<FKeyType, uint16> KeyToPropertyIndex;
	/** When true, KeyToPropertyIndex contains invalid entries which must be purged before use */
	bool bLUTContainsInvalidEntries = false;
};


} // namespace MovieScene
} // namespace UE

